# Doni KV Store

**Doni** is a simple, high-performance, TCP-based key-value (KV) store written in Rust. It supports storing and retrieving arbitrary byte sequences, making it versatile for various applications. The server employs a custom protocol over TCP, ensuring efficient communication and secure access through environment-based authentication.



## Table of Contents

- [Doni KV Store](#doni-kv-store)
  - [Table of Contents](#table-of-contents)
  - [Features](#features)
  - [Requirements](#requirements)
  - [Installation](#installation)
  - [Configuration](#configuration)
    - [Environment Variables](#environment-variables)
    - [Setting Up `.env` File](#setting-up-env-file)
  - [Usage](#usage)
    - [Starting the Server](#starting-the-server)
    - [Client Interaction](#client-interaction)
      - [Command Structure](#command-structure)
      - [PUT Command](#put-command)
      - [GET Command](#get-command)
      - [DELETE Command](#delete-command)
  - [Protocol Specification](#protocol-specification)
    - [General Command Format](#general-command-format)
    - [Supported Commands](#supported-commands)
    - [Response Format](#response-format)
  - [Testing](#testing)
    - [Using Netcat (`nc`)](#using-netcat-nc)
    - [Using a Custom Python Client](#using-a-custom-python-client)
      - [Python Client Example](#python-client-example)
      - [Instructions](#instructions)
      - [Expected Output](#expected-output)
  - [Error Handling](#error-handling)
    - [Common Errors](#common-errors)
    - [Example Scenarios](#example-scenarios)
  - [Concurrency and Performance](#concurrency-and-performance)
    - [Key Points](#key-points)
  - [Contributing](#contributing)
    - [Steps to Contribute](#steps-to-contribute)
  - [License](#license)
  - [Acknowledgements](#acknowledgements)
  - [Contact](#contact)



## Features

- **TCP-Based Communication**: Operates over TCP/IP, supporting both IPv4 and IPv6.
- **Custom Protocol**: Tailored command structure for efficient key-value operations.
- **Arbitrary Byte Support**: Store and retrieve values as raw byte sequences, including data with embedded newlines or carriage returns.
- **Authentication**: Secure access using a secret key defined via environment variables.
- **Concurrency**: Handles multiple client connections simultaneously using asynchronous I/O.
- **In-Memory Storage**: Fast access to key-value pairs stored in memory.
- **Extensible**: Easily adaptable for additional features like data persistence or advanced authentication mechanisms.



## Requirements

- **Rust**: Version 1.60 or later.
- **Cargo**: Rust’s package manager (comes bundled with Rust).
- **Environment**: Ability to set environment variables or use a `.env` file.



## Installation

1. **Clone the Repository**

   ```bash
   git clone https://github.com/yourusername/doni-kv-store.git
   cd doni-kv-store
   ```

2. **Build the Project**

   Ensure you have Rust and Cargo installed. If not, install them from [rustup.rs](https://rustup.rs/).

   ```bash
   cargo build --release
   ```

   The compiled binary will be located at `target/release/doni`.



## Configuration

Doni uses environment variables to configure its operation. These can be set directly in your environment or defined in a `.env` file located at the root of the project.

### Environment Variables

| Variable          | Description                                        | Default Value   | Required |
|-|-|--|-|
| `DONI_SECRET_KEY` | The secret key for authenticating clients.         | *None*          | Yes      |
| `DONI_HOST`       | The host address to bind the server.               | `127.0.0.1`     | No       |
| `DONI_PORT`       | The port number on which the server listens.       | *None*          | Yes      |

### Setting Up `.env` File

Create a `.env` file in the root directory of the project with the following content:

```env
DONI_SECRET_KEY=your_secret_key
DONI_HOST=127.0.0.1
DONI_PORT=8080
```

**Note**: Replace `your_secret_key` with a strong, unpredictable string to ensure secure authentication.



## Usage

### Starting the Server

After configuring the environment variables, you can start the Doni KV Store server.

1. **Ensure `.env` is Configured**

   Ensure that the `.env` file is correctly set up with `DONI_SECRET_KEY`, `DONI_HOST`, and `DONI_PORT`.

2. **Run the Server**

   ```bash
   cargo run --release
   ```

   **Output:**

   ```
   Doni running on 127.0.0.1:8080
   ```

   The server is now listening for client connections on the specified host and port.

### Client Interaction

Clients communicate with Doni using a custom protocol over TCP. Each command is a single-line message terminated by a newline character (`\n`). The server responds with status messages, and in some cases, additional data.

#### Command Structure

```
<AUTH_KEY> <COMMAND> <KEY> [<VALUE_LENGTH>]\n
<VALUE_BYTES>
```

- `<AUTH_KEY>`: Secret key for authentication.
- `<COMMAND>`: One of `PUT`, `GET`, or `DELETE`.
- `<KEY>`: The key for the operation.
- `<VALUE_LENGTH>`: (Required for `PUT`) The length of the value in bytes.
- `<VALUE_BYTES>`: (Required for `PUT`) The raw byte sequence to store.



#### PUT Command

**Description**: Stores a key-value pair where the value can be an arbitrary byte sequence.

**Format**:

```
<AUTH_KEY> PUT <KEY> <VALUE_LENGTH>\n
<VALUE_BYTES>
```

- **`<VALUE_LENGTH>`**: The exact number of bytes in `<VALUE_BYTES>`.

**Example**:

Store the key `username` with the value `alice`.

```
my_secret_key PUT username 5
alice
```

**Server Response**:

```
OK
```

**Handling Binary Data**:

When storing binary data, ensure that `<VALUE_LENGTH>` accurately reflects the number of bytes being sent.



#### GET Command

**Description**: Retrieves the value associated with a given key.

**Format**:

```
<AUTH_KEY> GET <KEY>\n
```

**Example**:

Retrieve the value for the key `username`.

```
my_secret_key GET username
```

**Server Response**:

- **Success**:

  ```
  OK <VALUE_LENGTH>\n
  <VALUE_BYTES>
  ```

  Example:

  ```
  OK 5
  alice
  ```

- **Key Not Found**:

  ```
  ERROR KeyNotFound
  ```



#### DELETE Command

**Description**: Removes the key-value pair from the store.

**Format**:

```
<AUTH_KEY> DELETE <KEY>\n
```

**Example**:

Delete the key `username`.

```
my_secret_key DELETE username
```

**Server Response**:

- **Success**:

  ```
  OK
  ```

- **Key Not Found**:

  ```
  ERROR KeyNotFound
  ```



## Protocol Specification

### General Command Format

Each command consists of a single line terminated by a newline character (`\n`). Commands may be followed by raw byte data as specified.

```
<AUTH_KEY> <COMMAND> <KEY> [<VALUE_LENGTH>]\n
<VALUE_BYTES>
```

- **Authentication**: The client must include the correct `AUTH_KEY` with each request. Requests with invalid or missing keys are rejected.

### Supported Commands

1. **PUT**

   - **Purpose**: Inserts or updates a key-value pair.
   - **Format**:
     ```
     <AUTH_KEY> PUT <KEY> <VALUE_LENGTH>\n
     <VALUE_BYTES>
     ```
   - **Response**:
     - `OK` on success.
     - `ERROR <message>` on failure.

2. **GET**

   - **Purpose**: Retrieves the value for a specified key.
   - **Format**:
     ```
     <AUTH_KEY> GET <KEY>\n
     ```
   - **Response**:
     - `OK <VALUE_LENGTH>\n<VALUE_BYTES>` on success.
     - `ERROR KeyNotFound` if the key does not exist.

3. **DELETE**

   - **Purpose**: Removes a key-value pair.
   - **Format**:
     ```
     <AUTH_KEY> DELETE <KEY>\n
     ```
   - **Response**:
     - `OK` on success.
     - `ERROR KeyNotFound` if the key does not exist.

### Response Format

Responses from the server are structured as follows:

```
<STATUS> [<DATA>]
```

- `<STATUS>`: `OK` for successful operations or `ERROR <message>` for failures.
- `<DATA>`: Present only in `GET` responses, indicating the length and the raw bytes of the value.



## Testing

### Using Netcat (`nc`)

**Note**: `netcat` is suitable for basic testing but has limitations with binary data.

1. **Connect to the Server**

   ```bash
   nc 127.0.0.1 8080
   ```

2. **Perform a PUT Operation**

   ```
   your_secret_key PUT testKey 11
   Hello World
   ```

   **Explanation**:
   - `11` is the length of `Hello World`.

   **Server Response**:

   ```
   OK
   ```

3. **Perform a GET Operation**

   ```
   your_secret_key GET testKey
   ```

   **Server Response**:

   ```
   OK 11
   Hello World
   ```

4. **Perform a DELETE Operation**

   ```
   your_secret_key DELETE testKey
   ```

   **Server Response**:

   ```
   OK
   ```

5. **GET After DELETE**

   ```
   your_secret_key GET testKey
   ```

   **Server Response**:

   ```
   ERROR KeyNotFound
   ```

**Limitations**:
- Handling binary data with `netcat` is cumbersome.
- Newlines in `<VALUE_BYTES>` can disrupt the protocol.



### Using a Custom Python Client

For comprehensive testing, especially with arbitrary byte sequences, use a custom client script.

#### Python Client Example

```python
import socket

def send_put(sock, auth_key, key, value_bytes):
    value_length = len(value_bytes)
    command = f"{auth_key} PUT {key} {value_length}\n".encode()
    sock.sendall(command)
    sock.sendall(value_bytes)
    response = recv_line(sock)
    print(f"PUT Response: {response}")

def send_get(sock, auth_key, key):
    command = f"{auth_key} GET {key}\n".encode()
    sock.sendall(command)
    header = recv_line(sock)
    if header.startswith(b"OK "):
        _, length_str = header.split(b' ', 1)
        value_length = int(length_str)
        value_bytes = recv_exact(sock, value_length)
        print(f"GET Response: OK {value_length} bytes received")
        print(f"Value Bytes: {value_bytes}")
    else:
        print(f"GET Response: {header}")

def send_delete(sock, auth_key, key):
    command = f"{auth_key} DELETE {key}\n".encode()
    sock.sendall(command)
    response = recv_line(sock)
    print(f"DELETE Response: {response}")

def recv_line(sock):
    line = b''
    while not line.endswith(b'\n'):
        char = sock.recv(1)
        if not char:
            break
        line += char
    return line.strip()

def recv_exact(sock, n):
    data = b''
    while len(data) < n:
        chunk = sock.recv(n - len(data))
        if not chunk:
            break
        data += chunk
    return data

def main():
    HOST = '127.0.0.1'
    PORT = 8080
    AUTH_KEY = 'your_secret_key'  # Replace with your actual secret key

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
        sock.connect((HOST, PORT))
        
        # Example PUT with arbitrary bytes
        key = 'binaryKey'
        value = b'This is a test value with \n newlines and \r carriage returns.'
        send_put(sock, AUTH_KEY, key, value)

        # Example GET
        send_get(sock, key, key)

        # Example DELETE
        send_delete(sock, AUTH_KEY, key)

        # Example GET after DELETE
        send_get(sock, AUTH_KEY, key)

if __name__ == "__main__":
    main()
```

#### Instructions

1. **Save the Script**

   Save the above Python script as `client.py`.

2. **Update Secret Key**

   Replace `'your_secret_key'` with the actual secret key set in your `.env` file.

3. **Run the Client**

   ```bash
   python3 client.py
   ```

#### Expected Output

```
PUT Response: OK
GET Response: OK 51 bytes received
Value Bytes: b'This is a test value with \n newlines and \r carriage returns.'
DELETE Response: OK
GET Response: ERROR KeyNotFound
```



## Error Handling

Doni provides meaningful error messages to help clients understand the nature of any issues encountered during operations.

### Common Errors

| Error Message           | Description                                                   |
|-||
| `ERROR AuthenticationFailed` | The provided authentication key does not match the server's key. |
| `ERROR InvalidCommand`         | The command is malformed or missing required fields.            |
| `ERROR KeyNotFound`            | The specified key does not exist in the store.                  |
| `ERROR IncompleteValue`        | The provided `<VALUE_BYTES>` does not match `<VALUE_LENGTH>`.    |

### Example Scenarios

1. **Invalid Authentication Key**

   **Client Request**:

   ```
   wrong_key GET username
   ```

   **Server Response**:

   ```
   ERROR AuthenticationFailed
   ```

2. **Malformed Command**

   **Client Request**:

   ```
   your_secret_key PUT incomplete_command
   ```

   **Server Response**:

   ```
   ERROR InvalidCommand
   ```

3. **Key Not Found**

   **Client Request**:

   ```
   your_secret_key GET unknownKey
   ```

   **Server Response**:

   ```
   ERROR KeyNotFound
   ```

4. **Incomplete Value Data**

   **Client Request**:

   ```
   your_secret_key PUT testKey 10
   short
   ```

   **Server Response**:

   ```
   ERROR IncompleteValue
   ```



## Concurrency and Performance

Doni leverages Rust's asynchronous capabilities through the Tokio runtime and uses `DashMap` for thread-safe, concurrent access to the in-memory key-value store. This design ensures that the server can handle multiple client connections simultaneously without significant performance degradation.

### Key Points

- **Asynchronous I/O**: Utilizes Tokio for efficient non-blocking operations.
- **Thread-Safe Storage**: `DashMap` allows concurrent reads and writes without explicit locking.
- **Scalability**: Capable of handling numerous simultaneous connections, limited primarily by system resources.



## Contributing

Contributions are welcome! Whether it's bug fixes, feature enhancements, or documentation improvements, your input helps make Doni better.

### Steps to Contribute

1. **Fork the Repository**

   Click the "Fork" button at the top-right of this repository's page.

2. **Clone Your Fork**

   ```bash
   git clone https://github.com/yourusername/doni-kv-store.git
   cd doni-kv-store
   ```

3. **Create a New Branch**

   ```bash
   git checkout -b feature/your-feature-name
   ```

4. **Make Changes**

   Implement your changes with clear commit messages.

5. **Commit and Push**

   ```bash
   git add .
   git commit -m "Add: Description of your changes"
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request**

   Navigate to the original repository and click on "Compare & pull request" to submit your changes for review.



## License

This project is licensed under the [MIT License](LICENSE).



## Acknowledgements

- **Rust Programming Language**: [https://www.rust-lang.org/](https://www.rust-lang.org/)
- **Tokio Runtime**: [https://tokio.rs/](https://tokio.rs/)
- **DashMap**: [https://github.com/xacrimon/dashmap](https://github.com/xacrimon/dashmap)
- **dotenvy**: [https://github.com/dotenvy-rs/dotenvy](https://github.com/dotenvy-rs/dotenvy)



## Contact

For any questions, issues, or suggestions, please open an issue in the [GitHub repository](https://github.com/maaxleq/doni/issues) or contact the maintainer at [maxenceleq@gmail.com](mailto:maxenceleq@gmail.com).



**Happy Key-Value Storing with Doni! 🚀**