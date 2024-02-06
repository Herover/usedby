Small linux utility to identify why a port is being used, and which process is listening on it.

Usage: compile and run `why 8080`, or just `cargo run 8080`. It will print something like

```
PID      EXE                        CMD                       
61473    /usr/bin/nc.openbsd        nc -l 8080 -v -v          
2639     /usr/bin/bash              /bin/bash                 
2500     /usr/bin/konsole           /usr/bin/konsole -session 1073696c6f000165851079800000023940020_1707171200_4252
2294     /usr/bin/ksmserver         /usr/bin/ksmserver        
1490     /usr/lib/systemd/systemd   /lib/systemd/systemd --user
1        ?                          /sbin/init splash         
```

In this example the process `nc` (netcat) with the process id 61473 is listening on port 8080, it was started from a bash terminal running in Konsole on a KDE system with systemd.

The first line explains columns, second line shows the process listening on port 8080, following lines has each parent process until there are no more parents or the user has unsufficient permissions to read information about the process.
