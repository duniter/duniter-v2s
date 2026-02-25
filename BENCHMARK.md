# Benchmark reference materials

This document describes in detail the equipment used to perform the benchmark of the runtime.

## runtime-1000 - 2026-02-23

The reference machine is a `Raspberry Pi 4 Model B Rev 1.4` with 8GB of RAM and an SSD connected via USB3.

### CPU

```
# lscpu
Architecture:                aarch64
  CPU op-mode(s):            32-bit, 64-bit
  Byte Order:                Little Endian
CPU(s):                      4
  On-line CPU(s) list:       0-3
Vendor ID:                   ARM
  Model name:                Cortex-A72
    Model:                   3
    Thread(s) per core:      1
    Core(s) per cluster:     4
    Socket(s):               -
    Cluster(s):              1
    Stepping:                r0p3
    CPU(s) scaling MHz:      67%
    CPU max MHz:             1500,0000
    CPU min MHz:             600,0000
    BogoMIPS:                108,00
    Flags:                   fp asimd evtstrm crc32 cpuid
Caches (sum of all):
  L1d:                       128 KiB (4 instances)
  L1i:                       192 KiB (4 instances)
  L2:                        1 MiB (1 instance)
NUMA:
  NUMA node(s):              1
  NUMA node0 CPU(s):         0-3
```

### SSD

```
# hdparm -I /dev/sda
/dev/sda:

ATA device, with non-removable media
        Model Number:       Samsung SSD 860 EVO M.2 1TB
        Serial Number:      XXXXXXXXXXXXXXX
        Firmware Revision:  RVT24B6Q
        Transport:          Serial, ATA8-AST, SATA 1.0a, SATA II Extensions, SATA Rev 2.5, SATA Rev 2.6, SATA Rev 3.0
Standards:
        Used: unknown (minor revision code 0x005e)
        Supported: 11 8 7 6 5
        Likely used: 11
Configuration:
        Logical         max     current
        cylinders       16383   16383
        heads           16      16
        sectors/track   63      63
        --
        CHS current addressable sectors:    16514064
        LBA    user addressable sectors:   268435455
        LBA48  user addressable sectors:  1953525168
        Logical  Sector size:                   512 bytes
        Physical Sector size:                   512 bytes
        Logical Sector-0 offset:                  0 bytes
        device size with M = 1024*1024:      953869 MBytes
        device size with M = 1000*1000:     1000204 MBytes (1000 GB)
        cache/buffer size  = unknown
        Form Factor: unknown (0x0007]
        Nominal Media Rotation Rate: Solid State Device
Capabilities:
        LBA, IORDY(can be disabled)
        Queue depth: 32
        Standby timer values: spec'd by Standard, no device specific minimum
        R/W multiple sector transfer: Max = 1   Current = 1
        DMA: mdma0 mdma1 mdma2 udma0 udma1 udma2 udma3 udma4 udma5 *udma6
             Cycle time: min=120ns recommended=120ns
        PIO: pio0 pio1 pio2 pio3 pio4
             Cycle time: no flow control=120ns  IORDY flow control=120ns
Commands/features:
        Enabled Supported:
           *    SMART feature set
                Security Mode feature set
           *    Power Management feature set
           *    Write cache
           *    Look-ahead
           *    Host Protected Area feature set
           *    WRITE_BUFFER command
           *    READ_BUFFER command
           *    NOP cmd
           *    DOWNLOAD_MICROCODE
                SET_MAX security extension
           *    48-bit Address feature set
           *    Device Configuration Overlay feature set
           *    Mandatory FLUSH_CACHE
           *    FLUSH_CACHE_EXT
           *    SMART error logging
           *    SMART self-test
           *    General Purpose Logging feature set
           *    WRITE_{DMA|MULTIPLE}_FUA_EXT
           *    64-bit World wide name
                Write-Read-Verify feature set
           *    WRITE_UNCORRECTABLE_EXT command
           *    {READ,WRITE}_DMA_EXT_GPL commands
           *    Segmented DOWNLOAD_MICROCODE
           *    Gen1 signaling speed (1.5Gb/s)
           *    Gen2 signaling speed (3.0Gb/s)
           *    Gen3 signaling speed (6.0Gb/s)
           *    Native Command Queueing (NCQ)
           *    Phy event counters
           *    READ_LOG_DMA_EXT equivalent to READ_LOG_EXT
                DMA Setup Auto-Activate optimization
                Device-initiated interface power management
           *    Asynchronous notification (eg. media change)
           *    Software settings preservation
                Device Sleep (DEVSLP)
           *    SMART Command Transport (SCT) feature set
           *    SCT Write Same (AC2)
           *    SCT Error Recovery Control (AC3)
           *    SCT Features Control (AC4)
           *    SCT Data Tables (AC5)
           *    Device encrypts all user data
           *    DOWNLOAD MICROCODE DMA command
           *    SET MAX SETPASSWORD/UNLOCK DMA commands
           *    WRITE BUFFER DMA command
           *    READ BUFFER DMA command
           *    Data Set Management TRIM supported (limit 8 blocks)
           *    Deterministic read ZEROs after TRIM
```

### USB

```
# lspci
00:00.0 PCI bridge: Broadcom Inc. and subsidiaries BCM2711 PCIe Bridge (rev 10)
01:00.0 USB controller: VIA Technologies, Inc. VL805/806 xHCI USB 3.0 Controller (rev 01)
# lspci -s 01: -v
01:00.0 USB controller: VIA Technologies, Inc. VL805/806 xHCI USB 3.0 Controller (rev 01) (prog-if 30 [XHCI])
        Subsystem: VIA Technologies, Inc. VL805/806 xHCI USB 3.0 Controller
        Device tree node: /sys/firmware/devicetree/base/scb/pcie@7d500000/pci@0,0/usb@0,0
        Flags: bus master, fast devsel, latency 0, IRQ 38
        Memory at 600000000 (64-bit, non-prefetchable) [size=4K]
        Capabilities: [80] Power Management version 3
        Capabilities: [90] MSI: Enable+ Count=4/4 Maskable- 64bit+
        Capabilities: [c4] Express Endpoint, IntMsgNum 0
        Capabilities: [100] Advanced Error Reporting
        Kernel driver in use: xhci_hcd
        Kernel modules: xhci_pci
# lsusb -t
/:  Bus 001.Port 001: Dev 001, Class=root_hub, Driver=xhci_hcd/1p, 480M
    |__ Port 001: Dev 002, If 0, Class=Hub, Driver=hub/4p, 480M
/:  Bus 002.Port 001: Dev 001, Class=root_hub, Driver=xhci_hcd/4p, 5000M
    |__ Port 002: Dev 002, If 0, Class=Mass Storage, Driver=uas, 5000M
# lsusb -s 002:002 -v
Bus 002 Device 002: ID 174c:1156 ASMedia Technology Inc. Forty
Negotiated speed: SuperSpeed (5Gbps)
Device Descriptor:
  bLength                18
  bDescriptorType         1
  bcdUSB               3.10
  bDeviceClass            0 [unknown]
  bDeviceSubClass         0 [unknown]
  bDeviceProtocol         0
  bMaxPacketSize0         9
  idVendor           0x174c ASMedia Technology Inc.
  idProduct          0x1156 Forty
  bcdDevice            1.00
  iManufacturer           2 Argon
  iProduct                3 Forty
  iSerial                 1 0000000000E4
  bNumConfigurations      1
  Configuration Descriptor:
    bLength                 9
    bDescriptorType         2
    wTotalLength       0x0079
    bNumInterfaces          1
    bConfigurationValue     1
    iConfiguration          0
    bmAttributes         0xc0
      Self Powered
    MaxPower                0mA
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       0
      bNumEndpoints           2
      bInterfaceClass         8 Mass Storage
      bInterfaceSubClass      6 SCSI
      bInterfaceProtocol     80 Bulk-Only
      iInterface              0
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x81  EP 1 IN
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0400  1x 1024 bytes
        bInterval               0
        bMaxBurst              15
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x02  EP 2 OUT
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0400  1x 1024 bytes
        bInterval               0
        bMaxBurst              15
    Interface Descriptor:
      bLength                 9
      bDescriptorType         4
      bInterfaceNumber        0
      bAlternateSetting       1
      bNumEndpoints           4
      bInterfaceClass         8 Mass Storage
      bInterfaceSubClass      6 SCSI
      bInterfaceProtocol     98
      iInterface              0
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x81  EP 1 IN
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0400  1x 1024 bytes
        bInterval               0
        bMaxBurst              15
        MaxStreams             32
        Data-in pipe (0x03)
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x02  EP 2 OUT
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0400  1x 1024 bytes
        bInterval               0
        bMaxBurst              15
        MaxStreams             32
        Data-out pipe (0x04)
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x83  EP 3 IN
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0400  1x 1024 bytes
        bInterval               0
        bMaxBurst              15
        MaxStreams             32
        Status pipe (0x02)
      Endpoint Descriptor:
        bLength                 7
        bDescriptorType         5
        bEndpointAddress     0x04  EP 4 OUT
        bmAttributes            2
          Transfer Type            Bulk
          Synch Type               None
          Usage Type               Data
        wMaxPacketSize     0x0400  1x 1024 bytes
        bInterval               0
        bMaxBurst               0
        Command pipe (0x01)
Binary Object Store Descriptor:
  bLength                 5
  bDescriptorType        15
  wTotalLength       0x002a
  bNumDeviceCaps          3
  USB 2.0 Extension Device Capability:
    bLength                 7
    bDescriptorType        16
    bDevCapabilityType      2
    bmAttributes   0x0000f41e
      BESL Link Power Management (LPM) Supported
    BESL value     1024 us
    Deep BESL value    61440 us
  SuperSpeed USB Device Capability:
    bLength                10
    bDescriptorType        16
    bDevCapabilityType      3
    bmAttributes         0x00
    wSpeedsSupported   0x000e
      Device can operate at Full Speed (12Mbps)
      Device can operate at High Speed (480Mbps)
      Device can operate at SuperSpeed (5Gbps)
    bFunctionalitySupport   1
      Lowest fully-functional device speed is Full Speed (12Mbps)
    bU1DevExitLat          10 micro seconds
    bU2DevExitLat        2047 micro seconds
  SuperSpeedPlus USB Device Capability:
    bLength                20
    bDescriptorType        16
    bDevCapabilityType     10
    bmAttributes         0x00000001
      Sublink Speed Attribute count 2
      Sublink Speed ID count 1
    wFunctionalitySupport   0x1100
      Min functional Speed Attribute ID: 0
      Min functional RX lanes: 1
      Min functional TX lanes: 1
    bmSublinkSpeedAttr[0]   0x000a4030
      Speed Attribute ID: 0 10Gb/s Symmetric RX SuperSpeedPlus
    bmSublinkSpeedAttr[1]   0x000a40b0
      Speed Attribute ID: 0 10Gb/s Symmetric TX SuperSpeedPlus
Device Status:     0x000d
  Self Powered
  U1 Enabled
  U2 Enabled
```
