proto/SimpleStructure.proto: feature_definitions/PrintingControl.xml sila_base/xslt/fdl2proto.xsl
	mkdir -p proto
	xsltproc \
	  sila_base/xslt/fdl2proto.xsl \
	  feature_definitions/PrintingControl.xml \
	  > proto/SimpleStructure.proto
