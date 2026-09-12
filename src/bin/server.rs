use std::{path::PathBuf, pin::Pin};

use sila_printer::get_printer;
use tokio::io::AsyncWriteExt;
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};

use tonic::{Request, Response, Status, transport::Server};
use uuid::Uuid;

use crate::pb::sila2::org::silastandard::{
    CommandConfirmation, CommandExecutionUuid, CreateBinaryRequest, CreateBinaryResponse,
    DeleteBinaryRequest, DeleteBinaryResponse, ExecutionInfo, String as SiLaString,
    UploadChunkRequest, UploadChunkResponse,
    binary_upload_server::BinaryUpload,
    printer::silaprintingcontrol::v1::{
        FlushScanParameters, FlushScanResponses, PrintParameters, PrintResponses,
        ScanPageParameters, ScanPageResponses, SubscribeCurrentPrinterStatusParameters,
        SubscribeCurrentPrinterStatusResponses,
        si_la_printing_control_server::{SiLaPrintingControl, SiLaPrintingControlServer},
    },
};

// SiLAFramework.proto and SiLABinaryTransfer.proto share the package
// `sila2.org.silastandard`, so both land in this one module.

pub mod pb {
    pub mod sila2 {
        pub mod org {
            pub mod silastandard {
                // framework types land here
                tonic::include_proto!("sila2.org.silastandard");

                pub mod printer {
                    pub mod silaprintingcontrol {
                        pub mod v1 {
                            tonic::include_proto!(
                                "sila2.org.silastandard.printer.silaprintingcontrol.v1"
                            );
                        }
                    }
                }
            }
        }
    }
}

struct PrinterServer;

struct UploadServer {
    directory: PathBuf,
}

fn io_error(err: std::io::Error) -> Status {
    Status::internal(err.to_string())
}

#[tonic::async_trait]
impl BinaryUpload for UploadServer {
    type UploadChunkStream =
        Pin<Box<dyn Stream<Item = Result<UploadChunkResponse, Status>> + Send>>;

    async fn create_binary(
        &self,
        request: Request<CreateBinaryRequest>,
    ) -> Result<Response<CreateBinaryResponse>, Status> {
        let r = request.into_inner();

        if r.chunk_count == 0 {
            return Err(Status::invalid_argument(
                "chunk_count must be greater than 0",
            ));
        } else if r.binary_size == 0 {
            return Err(Status::invalid_argument(
                "binary_size must be greater than 0",
            ));
        } else {
            let uuid = uuid::Uuid::new_v4().to_string();
            tokio::fs::create_dir(self.directory.join(&uuid))
                .await
                .map_err(io_error)?;

            let metadata_file = self.directory.join(&uuid).join("metadata.json");
            let metadata =
                serde_json::to_string(&r).map_err(|e| Status::internal(e.to_string()))?;
            tokio::fs::write(&metadata_file, metadata)
                .await
                .map_err(io_error)?;

            let response = CreateBinaryResponse {
                binary_transfer_uuid: uuid,
                lifetime_of_binary: None,
            };
            Ok(Response::new(response))
        }
    }

    async fn delete_binary(
        &self,
        request: Request<DeleteBinaryRequest>,
    ) -> Result<Response<DeleteBinaryResponse>, Status> {
        let request = request.into_inner();
        let uuid = Uuid::parse_str(&request.binary_transfer_uuid)
            .map_err(|_| Status::invalid_argument("Invalid UUID"))?;
        let folder = self.directory.join(uuid.to_string());
        if !tokio::fs::try_exists(&folder).await.map_err(io_error)? {
            return Err(Status::not_found("Unknown upload"));
        }
        tokio::fs::remove_dir_all(&folder).await.map_err(io_error)?;
        Ok(Response::new(DeleteBinaryResponse {}))
    }

    async fn upload_chunk(
        &self,
        request: Request<tonic::Streaming<UploadChunkRequest>>,
    ) -> Result<Response<Self::UploadChunkStream>, Status> {
        let directory = self.directory.clone();
        let replies = request.into_inner().then(move |message| {
            let directory = directory.clone();
            async move {
                let chunk = message?;
                let uuid = Uuid::parse_str(&chunk.binary_transfer_uuid)
                    .map_err(|_| Status::invalid_argument("Invalid UUID"))?;

                let folder = directory.join(uuid.to_string());
                if !tokio::fs::try_exists(&folder).await.map_err(io_error)? {
                    return Err(Status::not_found("Unknown upload"));
                }

                let metadata_file = folder.join("metadata.json");
                let metadata_json = tokio::fs::read(&metadata_file).await.map_err(io_error)?;
                let metadata: CreateBinaryRequest = serde_json::from_slice(&metadata_json)
                    .map_err(|e| Status::internal(e.to_string()))?;

                if chunk.chunk_index >= metadata.chunk_count {
                    return Err(Status::internal("Chunk index out of range"));
                }

                let mut total_size: u64 = chunk.payload.len() as u64;
                for i in 0..chunk.chunk_index {
                    let size = tokio::fs::metadata(folder.join(i.to_string()))
                        .await
                        .map_err(|e| Status::internal(e.to_string()))?
                        .len();

                    total_size = total_size
                        .checked_add(size)
                        .ok_or_else(|| Status::invalid_argument("Upload size overflow"))?;
                }

                if total_size > metadata.binary_size {
                    return Err(Status::invalid_argument("Upload exceeds declared size"));
                }
                let mut file = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(folder.join(chunk.chunk_index.to_string()))
                    .await
                    .map_err(io_error)?;
                file.write_all(&chunk.payload).await.map_err(io_error)?;
                file.flush().await.map_err(io_error)?;
                Ok(UploadChunkResponse {
                    binary_transfer_uuid: chunk.binary_transfer_uuid,
                    chunk_index: chunk.chunk_index,
                    lifetime_of_binary: None,
                })
            }
        });
        Ok(Response::new(Box::pin(replies)))
    }
}

#[tonic::async_trait]
impl SiLaPrintingControl for PrinterServer {
    type Print_InfoStream = ReceiverStream<Result<ExecutionInfo, Status>>;
    type ScanPage_InfoStream = ReceiverStream<Result<ExecutionInfo, Status>>;
    type FlushScan_InfoStream = ReceiverStream<Result<ExecutionInfo, Status>>;
    type Subscribe_CurrentPrinterStatusStream =
        ReceiverStream<Result<SubscribeCurrentPrinterStatusResponses, Status>>;

    async fn subscribe_current_printer_status(
        &self,
        request: Request<SubscribeCurrentPrinterStatusParameters>,
    ) -> Result<Response<Self::Subscribe_CurrentPrinterStatusStream>, Status> {
        let r = request.into_inner();

        let (tx, rx) = tokio::sync::mpsc::channel(4);

        tokio::spawn(async move {
            loop {
                let printer = match get_printer() {
                    Ok(printer) => printer,
                    Err(error) => {
                        let _ = tx.send(Err(Status::unavailable(error.to_string()))).await;
                        break;
                    }
                };
                let response = SubscribeCurrentPrinterStatusResponses {
                    current_printer_status: Some(SiLaString {
                        value: printer.status.to_string(),
                    }),
                };

                if tx.send(Ok(response)).await.is_err() {
                    break;
                }

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Response::new(
            Self::Subscribe_CurrentPrinterStatusStream::new(rx),
        ))
    }

    async fn print(
        &self,
        request: Request<PrintParameters>,
    ) -> Result<Response<CommandConfirmation>, Status> {
        let request = request.into_inner();
        println!("{:?}", request);
        Ok(Response::new(CommandConfirmation {
            command_execution_uuid: None,
            lifetime_of_execution: None,
        }))
    }

    async fn print_info(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<Self::Print_InfoStream>, Status> {
        todo!()
    }

    async fn print_result(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<PrintResponses>, Status> {
        todo!()
    }

    async fn scan_page(
        &self,
        request: Request<ScanPageParameters>,
    ) -> Result<Response<CommandConfirmation>, Status> {
        todo!()
    }

    async fn scan_page_result(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<ScanPageResponses>, Status> {
        todo!()
    }

    async fn scan_page_info(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<Self::ScanPage_InfoStream>, Status> {
        todo!()
    }

    async fn flush_scan(
        &self,
        request: Request<FlushScanParameters>,
    ) -> Result<Response<CommandConfirmation>, Status> {
        todo!()
    }

    async fn flush_scan_result(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<FlushScanResponses>, Status> {
        todo!()
    }

    async fn flush_scan_info(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<Self::FlushScan_InfoStream>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(SiLaPrintingControlServer::new(PrinterServer))
        .serve(addr)
        .await?;
    Ok(())
}
