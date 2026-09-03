proto/PrintingControl.proto: feature_definitions/PrintingControl.xml sila_base/xslt/fdl2proto.xsl
	mkdir -p proto
	xsltproc \
	  sila_base/xslt/fdl2proto.xsl \
	  feature_definitions/PrintingControl.xml \
	  > proto/PrintingControl.proto


.PHONY: sila-printer
sila-printer: proto/PrintingControl.proto
	cargo build --bin sila-printer
