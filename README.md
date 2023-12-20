# epost

**epost** is a post-hoc analysis tool for equality saturation with egg.

It provides profiling of an egg run.


**epost-egg-record**:

The part should be running the equality saturation and then save the info logs into stdin

```
$ epost-egg-record 2> epost.data
```

Note that now this works by tweaking the egg by adding hooks to print out information. This is a hack; we should improve by
keeping the egg unchanged, adding logs to egg, and then turning out debugging mode here that epost reads the egg log
and extract the relevant information.

**epost-script**

Read and parse the data and then output the trace

```
$ epost-script epost.data -o epost.script
```

