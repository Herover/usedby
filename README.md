Small linux utility to identify what listens on a port or is using a file, and display information about it.

**Install** by running `cargo install usedby` - requires cargo from Rust.

**Example use**: the program comes with a handy help output that shows all options, here's some example uses:

Run `usedby port 8080`, or with the repository open `cargo run port 8080`. It will print something like

```
PID      UID      EXE                        CMD                       
120509   1000     /usr/bin/nc.openbsd        nc -v -v -l 8080          
2819     1000     /usr/bin/bash              /bin/bash                 
2575     1000     /usr/bin/konsole           /usr/bin/konsole -session 1073696c6f000165851079800000023940020_1707684371_75586
2339     1000     /usr/bin/ksmserver         /usr/bin/ksmserver        
1512     1000     /usr/lib/systemd/systemd   /lib/systemd/systemd --user
1        0        ?                          /sbin/init splash
```

In this example the process `nc` (netcat) with the process id 120509 is listening on port 8080, it was started from a bash terminal running in Konsole on a KDE system with systemd.

The first line explains columns, second line shows the process listening on port 8080, following lines has each parent process until there are no more parents or the user has unsufficient permissions to read information about the process.

Usedby can also display information about processes that is reading or writing a file. Ex.

```
$ usedby file ./test.txt
PID      UID      EXE                        CMD                       
121382   1000     /usr/bin/tail              tail -f test.txt          
2807     1000     /usr/bin/bash              /bin/bash                 
2575     1000     /usr/bin/konsole           /usr/bin/konsole -session 1073696c6f000165851079800000023940020_1707684371_75586
2339     1000     /usr/bin/ksmserver         /usr/bin/ksmserver        
1512     1000     /usr/lib/systemd/systemd   /lib/systemd/systemd --user
1        0        ?                          /sbin/init splash
```
