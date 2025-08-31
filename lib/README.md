# grapevine_lib

Grapevine backend is written totally seperately from the fronend.
This is entirely intentional, and technically means that one could,
without any changes to the backend itself, turn grapevine into a CLI
app.

To put this simply: this is very cool. To formalise this divide, the repository
is split into the two parts, and the backend is separated out here, as
`cargo` workspace.
