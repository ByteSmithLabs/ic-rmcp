## Deploy: 
```bash
dfx deploy <server_name> --argument '("YOUR_API_KEY")' --mode install
```

After deployment on local network or playground, you can access it at: `https://<CANISTER_ID>.icp0.io/mcp` (for playground) or `https://<CANISTER_ID>.localhost:<BINDING_PORT>/mcp` (for local).

You could directly use our trait handler to construct HTTP endpoint (see [counter](./counter/)) or use [`ic-http`](https://github.com/ByteSmithLabs/ic-http) (see [adder](./adder/)).
