# Notes

* The RP2040 samples the QSPI_SS line on boot. If this line is low, it will boot into its USB bootloader mode (i.e. enumerate as a mass storage device so you can upload new firmware). If this line is high on boot, it will start reading program instructions from the quad-SPI flash chip.

## Layout Notes

Just some measurements I've written down for my own designs are:
standard MX cutout - 14×14mm
distance between switches (center to center) - 19.05mm (both vertically and horizontally)
distance between caps - 0.95mm
distance between cap and case side - 0.475mm

Calculation example (60% case):
width of a case: 6 + (19.05 * 15) + (0.475 * 2) = 3mm bezel on each side, gap between case side and caps on each side, 15 keys in row.

Edit: 19.05mm distance between switches are only correct if they are two 1u keys. So for something like a 1.5u and 1u key distance between their centers would be (19.05 * 1.5)mm.

## TRRS Debug Connector

The footprint for the debug connector is a 3.5mm TRRS jack (part number PJ-320A, LCSC part number C2884926).

This footprint should be equivalent to the MJ-4PP-9 connector which is more easily found in Japan.
