use tokio_stream::wrappers::ReceiverStream;

use tonic::{Request, Response, Status, transport::Server};

use crate::pb::sila2::org::silastandard::{
    CommandConfirmation, CommandExecutionUuid, ExecutionInfo,
    printer::silaprintingcontrol::v1::{
        PrintParameters, PrintResponses, StartScanningParameters, StartScanningResponses,
        StopScanningParameters, StopScanningResponses, SubscribeCurrentPrinterStatusParameters,
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
    type StartScanning_InfoStream = ReceiverStream<Result<ExecutionInfo, Status>>;
    type StopScanning_InfoStream = ReceiverStream<Result<ExecutionInfo, Status>>;
    type Subscribe_CurrentPrinterStatusStream =
        ReceiverStream<Result<SubscribeCurrentPrinterStatusResponses, Status>>;

    async fn subscribe_current_printer_status(
        &self,
        request: Request<SubscribeCurrentPrinterStatusParameters>,
    ) -> Result<Response<Self::Subscribe_CurrentPrinterStatusStream>, Status> {
        todo!()
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

    async fn start_scanning(
        &self,
        request: Request<StartScanningParameters>,
    ) -> Result<Response<CommandConfirmation>, Status> {
        todo!()
    }

    async fn start_scanning_result(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<StartScanningResponses>, Status> {
        todo!()
    }

    async fn start_scanning_info(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<Self::StartScanning_InfoStream>, Status> {
        todo!()
    }

    async fn stop_scanning(
        &self,
        request: Request<StopScanningParameters>,
    ) -> Result<Response<CommandConfirmation>, Status> {
        todo!()
    }

    async fn stop_scanning_result(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<StopScanningResponses>, Status> {
        todo!()
    }

    async fn stop_scanning_info(
        &self,
        request: Request<CommandExecutionUuid>,
    ) -> Result<Response<Self::StopScanning_InfoStream>, Status> {
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
