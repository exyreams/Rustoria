# Rustoria: Hospital Management System


<div align="center">
  <pre style="color: cyan;">
      ██████╗░██╗░░░██╗░██████╗████████╗░█████╗░██████╗░██╗░█████╗░
      ██╔══██╗██║░░░██║██╔════╝╚══██╔══╝██╔══██╗██╔══██╗██║██╔══██╗
      ██████╔╝██║░░░██║╚█████╗░░░░██║░░░██║░░██║██████╔╝██║███████║
      ██╔══██╗██║░░░██║░╚═══██╗░░░██║░░░██║░░██║██╔══██╗██║██╔══██║
      ██║░░██║╚██████╔╝██████╔╝░░░██║░░░╚█████╔╝██║░░██║██║██║░░██║
      ╚═╝░░╚═╝░╚═════╝░╚═════╝░░░░╚═╝░░░░╚════╝░╚═╝░░╚═╝╚═╝╚═╝░░╚═╝
  </pre>

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.60+-orange.svg)](https://www.rust-lang.org/)
[![GitHub issues](https://img.shields.io/github/issues/exyreams/rustoria)](https://github.com/exyreams/rustoria/issues)
[![GitHub stars](https://img.shields.io/github/stars/exyreams/rustoria)](https://github.com/exyreams/rustoria/stargazers)


</div>

A comprehensive terminal-based application for hospital management built with Rust, featuring patient management, staff scheduling, medical records, and billing functionality.

## 🔍 Overview

Rustoria is a powerful, efficient terminal-based hospital management system built using Rust. It leverages the `ratatui` library for creating a responsive terminal UI and `rusqlite` for secure and reliable database operations. The system provides comprehensive tools for healthcare administrators to manage patients, staff, medical records, and billing in a single integrated application.

## ✨ Features

- **🧑‍⚕️ Patient Management**
  - Add, update, and delete patient profiles
  - View complete patient history and details
  - Search and filter patient records

- **👩‍⚕️ Staff Management**
  - Maintain staff records and credentials
  - Manage staff schedules and shift assignments
  - Track staff performance and specializations

- **📝 Medical Records**
  - Create and maintain detailed medical records
  - Attach test results and diagnosis information
  - Secure access controls for sensitive information

- **💰 Billing & Finance**
  - Generate and manage patient invoices
  - Track payments and outstanding balances
  - Generate financial reports

- **🔐 Authentication**
  - Secure password storage with bcrypt
  - Session management

## 📺 Demo

Use **`root`** as username/password (default credentials) or click "Create Account" to set up a new user.

- **Registration/Login:**

 https://github.com/user-attachments/assets/7b0e1c9a-40b6-48a7-bb66-1efe8d8287bc

- **Loading Mockdata to database:**

 https://github.com/user-attachments/assets/cf5d30ab-4e25-4dea-8b7f-0946aa361854

- **Biling & Finance Demo:**

https://github.com/user-attachments/assets/7a3234b0-794c-4951-8407-7a347ad2482f

- **Medical Records Management:**

https://github.com/user-attachments/assets/0ab8dcea-8d05-4ab7-992c-3f82ba7c95a9

- **Patient Management:**

https://github.com/user-attachments/assets/ece6bb80-4cf9-4b5f-bc99-465c77a11ad4

- **Staff management:**

https://github.com/user-attachments/assets/d0d82719-26f0-4156-83d3-8fff1a70cbb2


## 🏗️ Project Structure

```
Rustoria/
├── src/
│   ├── components/
│   │   ├── hospital/
│   │   │   ├── finance/
│   │   │   │   ├── invoice.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── update.rs
│   │   │   └── └── view.rs
│   │   │   ├── patients/
│   │   │   │   ├── add.rs
│   │   │   │   ├── delete.rs
│   │   │   │   ├── list.rs
│   │   │   │   ├── mod.rs
│   │   │   └── └── update.rs
│   │   │   ├── records/
│   │   │   │   ├── delete.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── retrieve.rs
│   │   │   │   ├── store.rs
│   │   │   └── └── update.rs
│   │   │   ├── staff/
│   │   │   │   ├── add.rs
│   │   │   │   ├── assign.rs
│   │   │   │   ├── delete.rs
│   │   │   │   ├── list.rs
│   │   │   │   ├── mod.rs
│   │   │   └── └── update.rs
│   │   └── └── mod.rs
│   │   ├── home.rs
│   │   ├── login.rs
│   │   ├── mod.rs
│   └── └── register.rs
│   ├── db/
│   │   ├── mod.rs
│   └── └── schema.sql
│   ├── app.rs
│   ├── auth.rs
│   ├── main.rs
│   ├── models.rs
│   ├── tui.rs
└── └── utils.rs
├── Cargo.toml
└── rustoria.db
```

## 🚀 Installation

### Prerequisites

- Rust 1.60 or newer
- Cargo package manager

### Setup

1. **Clone the repository**

```bash
git clone https://github.com/exyreams/Rustoria.git
cd Rustoria
```

2. **Install Rust (if not already installed)**

Visit [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) and follow the instructions for your platform.

3. **Build the project**

```bash
cargo build --release
```

4. **Database initialization**

The database is automatically created during the first run, using the schema defined in `src/db/schema.sql`.

## ▶️ Running the Application

From the project directory, run:

```bash
cargo run --release
```

Or directly execute the compiled binary:

```bash
./target/release/rustoria
```

## 📦 Dependencies

Rustoria relies on these key Rust crates:

- **[ratatui](https://github.com/ratatui-org/ratatui)**: Terminal UI library
- **[crossterm](https://github.com/crossterm-rs/crossterm)**: Terminal manipulation
- **[rusqlite](https://github.com/rusqlite/rusqlite)**: SQLite database interface
- **[bcrypt](https://github.com/Keats/rust-bcrypt)**: Password hashing
- **[anyhow](https://github.com/dtolnay/anyhow)**: Error handling
- **[serde](https://github.com/serde-rs/serde)**: Serialization framework
- **[time](https://github.com/time-rs/time)**: Time manipulation

## 🤝 Contributing

Contributions are welcome and appreciated! Here's how you can contribute:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

Please make sure your code follows the project's style conventions and includes appropriate tests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 👏 Acknowledgments

- Thanks to the Rust community for providing excellent libraries and tools
- Special appreciation to the `ratatui` team for their outstanding terminal UI framework
- All contributors who have helped improve this project

---

© 2025 Rustoria Hospital Management System
