# Domain Name System (DNS)

This repository is a clone of the software components in the DNS, namely a resolver and a name server.

## Resolver

The resolver runs on a client's computer. It's job is to get answers to the client's queries, which most frequently means getting the IP address of a specific host name. It's integrated into the OS so that calls to the C standard library name resolution functions made by the system pass through to the resolver.

## Name server

The name server runs on a server. It holds records for which it has authority. The most well known record is the A record, which holds an IP address assigned to the host name.
