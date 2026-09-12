use sila_printer::get_printer;
use tokio_stream::wrappers::ReceiverStream;

use tonic::{Request, Response, Status, transport::Server};

use crate::pb::sila2::org::silastandard::{
    CommandConfirmation, CommandExecutionUuid, ExecutionInfo, String as SiLaString,
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
        todo!()
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
