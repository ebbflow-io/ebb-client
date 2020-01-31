# `ebbflow-client`

**NOTE** This is `Beta` quality as of now, and could use some more features and configurable settings!

This is the end-host client for [`ebbflow`](https://ebbflow.io). This is used to proxy SSH or TCP connections between ebbflow and your local server. 

```
ebbflow --help
```

The client has two modes, `tcp` and `ssh`, and you select one of these modes when starting the client. Check out the help pages for each command by doing `ebbflow tcp --help` or `ebbflow ssh --help`.

**NOTE** By default, only `1` connection is proxied. This is intended to allow you to test without having tons of logs thrown at you.

## Getting Started

After following the [quickstart guide](https://preview.ebbflow.io/quickstart)

A quick way to test out ebbflow is to run the example code and point the client at that or to use the SSH feature.

For hosting and endpoint:
```
cargo run --example server &
cargo run -- tcp -c CERTFILE -k KEYFILE -p 8080 --dns YOURWEBSITE.COM &
curl YOURWEBSITE.COM
```

For SSHing:
```
cargo run -- ssh -c CERTFILE -k KEYFILE --accountid ACCTID &
ssh -J ACCTID@ebbflow.io USER@HOSTNAME
```

## Building & Testing

TODO

## Contributing 

TODO

## Future Plans

TODO

## How It Works / Design

TODO