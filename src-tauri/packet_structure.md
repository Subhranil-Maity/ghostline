# Packet Structure Specification

## Overview

All packets in the protocol follow a binary structure beginning with a **version byte**.
This allows future protocol changes while maintaining backward compatibility.

All multi-byte integers are encoded using **big-endian** byte order.

```
+---------+-------------+------------------+
| Version | Packet Type | Packet Payload   |
|  (u8)   |    (u8)     |    (variable)    |
+---------+-------------+------------------+
```

| Field       | Type     | Description               |
| ----------- | -------- | ------------------------- |
| Version     | u8       | Protocol version          |
| Packet Type | u8       | Indicates packet category |
| Payload     | variable | Depends on packet type    |

---

# Packet Types

| Type | Name     |
| ---- | -------- |
| `1`  | Event    |
| `2`  | Request  |
| `3`  | Response |

---

# Event Packet

```
+---------+-------------+--------------+--------------+-------------+
| Version | Packet Type | Event Type   | Message Len  | Message     |
|  u8     | u8 (=1)     | u8           | u32          | bytes       |
+---------+-------------+--------------+--------------+-------------+
```

### Event Types

| Value | Event       |
| ----- | ----------- |
| `1`   | ChatMessage |

### ChatMessage Event

```
Offset
0   Version
1   PacketType = 1
2   EventType  = 1
3   MessageLength (u32)
7   Message bytes (UTF-8)
```

Example layout:

```
+----+----+----+----+----+----+----+------------------+
| V  | 1  | 1  |         LEN         |   MESSAGE      |
+----+----+----+----+----+----+----+------------------+
```

---

# Request Packet

```
+---------+-------------+--------------+----------------+
| Version | Packet Type | Request ID   | Request Type   |
|  u8     | u8 (=2)     | u64          | u8             |
+---------+-------------+--------------+----------------+
```

### Request Types

| Value | Request         |
| ----- | --------------- |
| `1`   | GetCapabilities |

Example layout:

```
+----+----+------------------+----+
| V  | 2  |    RequestID     | 1  |
+----+----+------------------+----+
```

---

# Response Packet

```
+---------+-------------+--------------+----------------+
| Version | Packet Type | Request ID   | Response Type  |
|  u8     | u8 (=3)     | u64          | u8             |
+---------+-------------+--------------+----------------+
```

### Response Types

| Value | Response     |
| ----- | ------------ |
| `1`   | Capabilities |
| `2`   | Ok           |
| `3`   | Error        |

---

# Capabilities Response

```
+---------+-------------+--------------+--------------+--------------+
| Version | Packet Type | Request ID   | Resp Type=1  | Cap Count    |
|  u8     | u8 (=3)     | u64          | u8           | u32          |
+---------+-------------+--------------+--------------+--------------+
```

Then each capability entry:

```
+--------------+-------------------+
| String Len   | Capability Bytes  |
| u32          | UTF-8             |
+--------------+-------------------+
```

Example layout:

```
+----+----+----------+----+-----------+----+------+
| V  | 3  | ReqID    | 1  | CapCount  | ...caps... |
+----+----+----------+----+-----------+------------+
```

---

# OK Response

```
+---------+-------------+--------------+-------------+
| Version | Packet Type | Request ID   | Resp Type=2 |
|  u8     | u8 (=3)     | u64          | u8          |
+---------+-------------+--------------+-------------+
```

---

# Error Response

```
+---------+-------------+--------------+-------------+-------------+--------------+
| Version | Packet Type | Request ID   | Resp Type=3 | Msg Length  | Message      |
|  u8     | u8 (=3)     | u64          | u8          | u32         | UTF-8 bytes  |
+---------+-------------+--------------+-------------+-------------+--------------+
```

Example:

```
+----+----+----------+----+----+----------------+
| V  | 3  | ReqID    | 3  |LEN | ErrorMessage   |
+----+----+----------+----+----+----------------+
```

---

# Notes

* All strings are encoded as **UTF-8**.
* All integers use **big-endian encoding**.
* Length-prefixed fields use `u32`.
* The **Version field enables future protocol upgrades** without breaking compatibility.
* Unknown packet or subtype values should be treated as **invalid packets**.

---

# Example Packet (Chat Message)

Message: `"hello"`

```
Version:      01
PacketType:   01
EventType:    01
Length:       00 00 00 05
Message:      68 65 6c 6c 6f
```

Full packet:

```
01 01 01 00 00 00 05 68 65 6c 6c 6f
```

