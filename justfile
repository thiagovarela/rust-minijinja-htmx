ts := `date +%s`

db: 
    docker compose up -d postgres --remove-orphans

dev: db        
    cargo watch -c -x "run --bin server" -w server/src

format:
    cargo fmt
    npx prettier ./templates --write

css:
    npx tailwindcss -c tailwind.config.js -i ./main.css -o ./public/assets/platform.css --watch

run:
    cargo run -p server --release

deploy:
    cargo sqlx prepare --workspace
    fly deploy --local-only --push --image-label {{ts}}