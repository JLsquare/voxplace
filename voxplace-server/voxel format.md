# Voxel format .vxl

## Header

- "VXL " (4 bytes)
- name_length (1 byte)
- name (name_length * 1 byte)
- version (4 * 1 bytes)
- gzip (1 byte)
- palette_size (1 byte)
- grid_size (3 * 2 bytes)

## Palette (palette_size * 7 * 1 bytes)

- r (1 byte)
- g (1 byte)
- b (1 byte)
- a (1 byte)
- emission (1 byte)
- roughness (1 byte)
- metallic (1 byte)
- (repeat)

## Grid (grid_size * grid_size * grid_size * 1 byte and compressed if gzip is true)

- palette_index (1 byte)
- (repeat)