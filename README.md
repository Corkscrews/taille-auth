# Taille-Auth

Welcome to **Taille-Auth**, the initial server responsible for allowing Irish taxis to use a low-cost solution to authenticate and gain access to a suite of future _taille-server_ services. This project is written in [Rust](https://www.rust-lang.org/) using the [`actix-web`](https://actix.rs/) framework and is currently under active development.

## Overview

- **Purpose**: Provide a minimal authentication service to login and generate temporary JWT token pairs.  
- **Tech Stack**:  
  - [Rust](https://www.rust-lang.org/)  
  - [Actix-Web](https://actix.rs/)  
- **Database**: Not yet integrated (subject to change in future versions).  
- **Open Source**: The project remains open source to encourage community collaboration and transparency.

## Auth Flow

1. **Login**:  
   - Clients authenticate with credentials (to be defined) to obtain a short-lived JWT pair (access + refresh) tokens.  
2. **Protected Routes**:  
   - Once authenticated, these tokens allow clients to access other _taille-server_ services in the ecosystem.  

## Creating Users

- Currently, only clients with a valid `MASTER_KEY` bearer token can create new users.  
- In the future, this mechanism will be replaced with more secure service keys, ensuring a more refined and role-based approach for user management.

## Under Development

This project is still in the early stages:

- **No Database**: User data is currently handled in-memory or via static files. Future versions will integrate a database solution.  
- **Security Enhancements**: While JWTs and `MASTER_KEY` provide basic access control, future updates will introduce improved security measures.

## Getting Started

1. **Prerequisites**  
   - [Rust](https://www.rust-lang.org/tools/install) (stable release recommended)

2. **Clone the Repository**  
  ```bash
  git clone https://github.com/your-org/taille-auth.git
  cd taille-auth
  ```

3. **Build and Run**
  ```bash
  cargo run
  ```

4. **Run tests
  ```bash
  cargo test
  ```

## Contributing
Contributions are welcome! Feel free to open issues and pull requests. Check out our [contribution guidelines (coming soon)]() for more details.