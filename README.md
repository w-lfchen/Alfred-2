# Alfred 2

Discord bot acting as the successor to the beloved geekhub bot Alfred.

## Usage
1. Set the required [environment variables](#environment-variables). Using a `.env` file is supported.
2. Build the program.
3. Run the program.

## Environment Variables
- `DATABASE_URL`: Required by `sqlx` at build time to check queries; Not needed at runtime
- `BOT_TOKEN`: The Discord bot token
- `DOLPHIN_PATH`: Path to the file to read dolphin image urls from
- `STATE_DIR`: Directory where state should be saved

## Contributing
Contributions are always welcome.
Feel free to open issues or pull requests through GitHub or suggest features/fixes to me.
