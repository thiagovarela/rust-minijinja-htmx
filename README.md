## Rust + Minijinja + HTMX

### Just just...

```bash
cargo install just
```

```bash
cargo install cargo-watch
```

Well, tailwind JIT... ez
```bash
just css
```

Should spin a local postgres and run the server using cargo watch.
```bash
just dev
```

### Little setup

Given that I only implemented Google Sign In, there are some environment variables to set.
I use .cargo/config.toml when developing locally. 

```toml
RUST_LOG="debug"
GOOGLE_CLIENT_ID="some-google-client-id.apps.googleusercontent.com"
GOOGLE_CLIENT_SECRET="some-google-client-secret"
GOOGLE_REDIRECT_URI="http://localhost:3000/auth/callback/google"

DATABASE_URL="postgres://platform_user:platform_pwd@localhost/platform"

# openssl rand -hex 32
COOKIE_KEY="cookie-are-private"
JWT_KEY="jwt-also"
```