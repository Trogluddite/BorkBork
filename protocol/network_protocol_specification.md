# BorkBork Chat Service wire protocol
**VERSION: 0.0.1**
**09JULY2025**

## Overview
The BorkBork protocol is an application-layer definition of messages intended for use with chat servers and clients.
* Numerals are encoded as little-endian
* the first byte specifies the message type in all cases
* variable-length feilds are not null-terminated
* fixed-length fields ARE null terminated UNLESS the content is exactly the length of field

## Message Types

| Message Type | Value | Length |
| ------------ | ---- | ---------- |
| CHATMSG | 0 | variable |
| JOIN | 1 | variable |
| LEAVE | 2 | 1 byte |
| VERSION | 3 | 7 bytes |
| WELCOME | 4 | variable |
| EXTENDED | 5 | variable |

### CHATMSG
Sent by both client and server -- a variable length message whose content
represents a chat message.

Numeric ID of sender should map to a username. Username / ID mapping should be maintained by the server.
| Byte | Meaning | datatype hint |
| ------ | ------------------------------ | ----------------- |
| 0 | Type specifier, set to 0 | uint 8 |
| 1-2 | message length | uint 16 |
| 3-4 | numeric ID of sender | uint 16 |
| 5+ | message contents | char vector |

### JOIN
Sent by the client when joining the server. Expect Future expansion to
support authenticated users
| Byte | Meaning | datatype hint |
| ------ | ------------------------------ | ----------------- |
| 0 | type specifier, set to 1 | uint 8 |
| 1-2 | name length | uint 16 |
| 3+ | username | char vector |

### LEAVE
Sent by the client to indicate that they'd like to leave the server
May be expanded to include rooms, groupings, or other entities a user may be attached to
| Byte | Meaning | datatype hint |
| ------ | ------------------------------ | ----------------- |
| 0 | type specifier, set to 2 | uint 8 |

### VERSION
Sent by the server to indicate the server's semantic version
Clients may check this for compatibility
| Byte | Meaning | datatype hint |
| ------ | ------------------------------ | ----------------- |
| 0 | type specifier, set to 3 | uint 8 |
| 1-2 | Major revision | uint 16 |
| 3-4 | Minor Revision | uint 16 |
| 5-6 | Subminor Revision | uint 16 |

### WELCOME
Sent by the server after successfully negotiating a client connection.
| Byte | Meaning | datatype hint |
| ------ | ------------------------------ | ----------------- |
| 0 | type specifie, set to 4 | uint 8 |
| 1-2 | message length | uint 16 |
| 3+ | welcome message contents | char vector |

### EXTENDED
Reserved for future expansion past 256 types of message; may also be used for
experimental or non-defined use cases in custom clients and servers.

Extended message types, when used as part of the protocol, must have reserved type IDs
recorded as a part of the protocol
| Byte | Meaning | datatype hint |
| ------ | ------------------------------ | ----------------- |
| 0 | type specifier. Set to 256. | uint 8 |
| 1-4 | type specifier for extension | uint 64 |
| 5+ | content specific to extended message type | various |






