# ds-detect

The detection layer: find the hardware, decide which drivers to start.

## The problem

A driver image has to carry every driver the machine *might* need, because at
build time nobody knows which machine it will boot on. Starting them all is the
obvious answer and the wrong one: each driver probes registers and claims
resources, and a driver with no hardware either sits inert or - worse - gets
confused by a device that is not its own.

So the image carries everything and the system starts what it finds.

## The pipeline

```text
  image (all drivers + manifest)
       │
  scan      read PCI configuration space           -> what hardware exists
       │      hw/scan.rs
  match     manifest entry vs (vendor, device)     -> what could drive it
       │      manifest.rs
  plan      one driver per device, honour policy    -> what to start
       │      plan.rs
  calibrate read the BARs, check they are real     -> what to grant
       │      calibrate.rs
  start     load, grant, run
```

Each stage is a separate type because each has a different failure mode: a
scan can miss a device, a match can be too broad, a plan can drop a driver, and
a calibration can find a device that is present in the configuration space and
absent on the silicon. Keeping them apart is what makes each diagnosable from
a boot log.

## Match rules

A manifest entry claims a device by **either** an exact `(vendor, device)`
pair or a PCI class:

| rule | example | meaning |
|------|---------|---------|
| exact | `exact:8086:2415` | only that pair |
| class | `class:04` | every multimedia device |
| class | `class:0401` | every audio device |

An exact match is checked first, because it knows what it is talking to; a
class match is the fallback for hardware the build did not enumerate. Both are
tried, and the first entry that matches a device wins it - which is how one
device never ends up with two drivers.

## Load policy

| policy | when it starts |
|--------|----------------|
| `ondemand` | only when a scanned device matches (the default) |
| `always` | regardless, for drivers with no enumerable device |
| `manual` | never automatically; present in the image only |

## The number that matters

With seven drivers in the image and one sound card in the machine, one driver
starts. That is the whole point of the layer, and it is what the test
`a_machine_with_one_card_starts_one_driver_out_of_seven` pins down.

## Design notes

**No allocator.** The layer runs before anything is loaded, so every table is
`heapless::Vec` over inline storage. Sizes are fixed at build time and bounded
(`MAX_DEVICES`, `Manifest::MAX_ENTRIES`).

**The scan does not probe vendor registers.** It reads ids, class code and
BARs, and stops there. A driver that needs more does it itself after being
granted the mapping - probing unknown registers at scan time is how a
detection layer turns a harmless machine into a crashing one.

**BAR size is not required.** Reading a BAR's size means writing all-ones to
it, which is a write to hardware this layer does not own. A BAR counts as
usable when its *base* is non-zero, and the size is left for the driver or the
manager to fill in once it holds the resource.

**A failed calibration stops the driver.** A device that answers in the
configuration space but offers no usable BAR is reported in
`Detection::rejected` and not started, rather than being handed a mapping that
does not exist.

## Testing

```sh
cargo test -p ds-detect
```

The whole pipeline is tested against a `FakeBus` that is a table of
configuration registers, so a whole machine - an audio card, a network card,
two identical cards, a bus-bridge stub - can be described in a test with no
PCI bus present.

## Building the image

```sh
just driverimg-list      # show the catalogue without building
just driverimg           # build dist/TrangorgeOS-drivers-<version>.img
just detect-test         # run this crate's tests
```

`tools/mkdriverimg.sh` writes the manifest the loader reads and places it, with
the driver binaries, in a FAT16 partition in the image. See the script's header
for the layout.
