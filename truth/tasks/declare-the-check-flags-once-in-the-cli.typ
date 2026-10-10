#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Declare the check flags once in the CLI",
  status: done(2026, 10, 10),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

--refresh is declared twice, and run() special-cases schema then keeps a dead Some(Command::Schema) arm; a flattened CheckArgs with args\_conflicts\_with\_subcommands declares it once and matches every command in one place
