# Notes

* The RP2040 samples the QSPI_SS line on boot. If this line is low, it will boot into its USB bootloader mode (i.e. enumerate as a mass storage device so you can upload new firmware). If this line is high on boot, it will start reading program instructions from the quad-SPI flash chip.
