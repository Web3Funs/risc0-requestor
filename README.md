### Run

Run like this:
```
	./risc0-requestor -c xxxxxxxxxxx -i 600 -k xxxxxxxxxxxx -l 0.0.0.0:5678 -r http://127.0.0.1:6789

```
```
    -c, --contract <contract>    ZKPool demo contract [default: xxxxxxxxxxxxxx]
    -i, --interval <interval>    The interval time to send dummy task [default: 3600]
    -k, --key <key>              Set the private key to sign the blockchain request [default:]
    -l, --listen <listen>        Set the rpc server api endpoint [default: 0.0.0.0:5678]
    -r, --relayer <relayer>      The relayer rpc endpoint [default: http://127.0.0.1:6789]
```



