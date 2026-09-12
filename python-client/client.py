import grpc
import PrintingControl_pb2
from PrintingControl_pb2_grpc import SiLAPrintingControlStub

channel = grpc.insecure_channel("127.0.0.1:50051")
client = SiLAPrintingControlStub(channel)


stream = client.Subscribe_CurrentPrinterStatus(
    PrintingControl_pb2.Subscribe_CurrentPrinterStatus_Parameters()
)
for response in stream:
    print(response)
