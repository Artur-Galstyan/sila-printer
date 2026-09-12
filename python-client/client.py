from typing import cast

import grpc
import PrintingControl_pb2
from PrintingControl_pb2_grpc import SiLAPrintingControlStub
from SiLAFramework_pb2 import String

channel = grpc.insecure_channel("127.0.0.1:50051")
client = SiLAPrintingControlStub(channel)


stream = client.Subscribe_CurrentPrinterStatus(
    PrintingControl_pb2.Subscribe_CurrentPrinterStatus_Parameters()
)
for response in stream:
    response = cast(
        PrintingControl_pb2.Subscribe_CurrentPrinterStatus_Responses, response
    )
    print(response.CurrentPrinterStatus.value)
    if response.CurrentPrinterStatus.value == "idle":
        client.Print(PrintingControl_pb2.Print_Parameters(File=String(value="wtf")))
