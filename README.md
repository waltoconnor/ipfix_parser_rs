# About
This is an IPFIX parser that implements a subset of the [RFC 7011](https://www.rfc-editor.org/rfc/rfc7011.html) standard. 
This project is adapted from an IPFIX collector I wrote to handle the stream of data coming from a Broadscan dataplane hardware telemetry system, and in that scenario it was able to process in excess of 100,000K IPFIX packets per second while running on a 32C/64T Intel Xeon Gold 5218 CPU. As this project is targeting a specific device, it only supports features of that device for the time being  and certain things (namely variable length data and options templates) are not supported.

Currently supported are:
- IPFIX over UDP
- Parsing message headers
- Parsing Template sets
- Parsing Data sets
- Enterprise Numbers
- Tracking different ODIDs separately (both for templates and for data)

I have not worked on the information model yet, each row is stored with the field ID and enterprise number rather for now. All the values are stored as `u8`s, `u16`s, `u32`s, `u64`s, or `Vector<u8>` if the data does not align with an integral type.

# Result Format
Results are stored on a per-packet basis. The structure of the packets is as follows:
- PacketInfo
    - Export Time
    - Sequence Number
    - Number of unparseable sets
    - ODID
    - Vec\<Templates\>
        - ID
        - Vec\<TemplateFields\>
            - Width (Bytes)
            - Start Byte
            - Enterprise Number
            - Field ID
    - Vec\<DataSet\>
        - Template ID
        - Vec\<Fields\>
            - ID,EN => Data


# TODO
- There is currently no way to actually collect the aggregated data from the thread it lives in, this should only require a simple mutex or send scheme to implement.