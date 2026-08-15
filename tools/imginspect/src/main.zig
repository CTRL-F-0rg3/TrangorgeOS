const std = @import("std");

const SECTOR_SIZE: usize = 512;
const MAX_ENTRIES: usize = 64;

fn readU16(buf: []const u8, offset: usize) u16 {
    return std.mem.readInt(u16, buf[offset..][0..2], .little);
}

fn readU32(buf: []const u8, offset: usize) u32 {
    return std.mem.readInt(u32, buf[offset..][0..4], .little);
}

const Partition = struct {
    index: usize,
    part_type: u8,
    start_lba: u32,
    length_blocks: u32,
};

fn fsHint(part_type: u8) []const u8 {
    return switch (part_type) {
        0x0B, 0x0C, 0x0E => "FAT32",
        0x83 => "ext4 (hint)",
        else => "unknown",
    };
}

fn parseMbr(sector: []const u8, out: *[4]?Partition) bool {
    if (sector[510] != 0x55 or sector[511] != 0xAA) return false;

    var i: usize = 0;
    while (i < 4) : (i += 1) {
        const offset = 446 + i * 16;
        const part_type = sector[offset + 4];
        if (part_type == 0) {
            out[i] = null;
            continue;
        }
        out[i] = Partition{
            .index = i,
            .part_type = part_type,
            .start_lba = readU32(sector, offset + 8),
            .length_blocks = readU32(sector, offset + 12),
        };
    }
    return true;
}

const Fat32Info = struct {
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    fat_size_32: u32,
    total_sectors_32: u32,
    root_cluster: u32,
};

fn parseFat32(sector: []const u8) ?Fat32Info {
    if (sector[510] != 0x55 or sector[511] != 0xAA) return null;
    const bytes_per_sector = readU16(sector, 11);
    const fat_size_32 = readU32(sector, 36);
    if (bytes_per_sector != 512 or fat_size_32 == 0) return null;

    return Fat32Info{
        .bytes_per_sector = bytes_per_sector,
        .sectors_per_cluster = sector[13],
        .reserved_sectors = readU16(sector, 14),
        .num_fats = sector[16],
        .fat_size_32 = fat_size_32,
        .total_sectors_32 = readU32(sector, 32),
        .root_cluster = readU32(sector, 44),
    };
}

fn firstDataSector(info: Fat32Info) u32 {
    return @as(u32, info.reserved_sectors) + info.fat_size_32 * @as(u32, info.num_fats);
}

fn clusterToSector(info: Fat32Info, cluster: u32) u32 {
    return firstDataSector(info) + (cluster - 2) * @as(u32, info.sectors_per_cluster);
}

const DirEntry = struct {
    name: [11]u8,
    attr: u8,
    cluster: u32,
    size: u32,
};

fn parseDirEntries(sector: []const u8, entries: *[MAX_ENTRIES]DirEntry, count: *usize) bool {
    var i: usize = 0;
    while (i + 32 <= sector.len) : (i += 32) {
        const entry = sector[i .. i + 32];
        if (entry[0] == 0x00) return true;
        if (entry[0] == 0xE5) continue;
        if (entry[11] == 0x0F) continue;
        if (count.* >= MAX_ENTRIES) continue;

        const cluster_high: u32 = readU16(entry, 20);
        const cluster_low: u32 = readU16(entry, 26);
        entries[count.*] = DirEntry{
            .name = entry[0..11].*,
            .attr = entry[11],
            .cluster = (cluster_high << 16) | cluster_low,
            .size = readU32(entry, 28),
        };
        count.* += 1;
    }
    return false;
}

fn formatShortName(buf: []u8, raw: [11]u8) []const u8 {
    var len: usize = 0;
    for (raw[0..8]) |b| {
        if (b == ' ') break;
        buf[len] = b;
        len += 1;
    }
    if (raw[8] != ' ') {
        buf[len] = '.';
        len += 1;
        for (raw[8..11]) |b| {
            if (b == ' ') break;
            buf[len] = b;
            len += 1;
        }
    }
    return buf[0..len];
}

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const args = try std.process.argsAlloc(allocator);
    defer std.process.argsFree(allocator, args);

    if (args.len < 2) {
        std.debug.print("usage: imginspect <disk-image-path>\n", .{});
        return;
    }

    const path = args[1];
    const file = try std.fs.cwd().openFile(path, .{});
    defer file.close();

    var sector0: [SECTOR_SIZE]u8 = undefined;
    _ = try file.readAll(&sector0);

    var mounted_offset: u64 = 0;
    var fat32_sector: [SECTOR_SIZE]u8 = sector0;
    var partitions: [4]?Partition = .{ null, null, null, null };

    if (parseMbr(&sector0, &partitions)) {
        std.debug.print("MBR partition table found:\n", .{});
        for (partitions) |maybe_part| {
            if (maybe_part) |part| {
                std.debug.print(
                    "  [{d}] type=0x{X:0>2} ({s}) start_lba={d} length_blocks={d}\n",
                    .{ part.index, part.part_type, fsHint(part.part_type), part.start_lba, part.length_blocks },
                );
                if (mounted_offset == 0 and (part.part_type == 0x0B or part.part_type == 0x0C or part.part_type == 0x0E)) {
                    mounted_offset = @as(u64, part.start_lba) * SECTOR_SIZE;
                }
            }
        }
        if (mounted_offset != 0) {
            try file.seekTo(mounted_offset);
            _ = try file.readAll(&fat32_sector);
        }
    } else {
        std.debug.print("no MBR signature at sector 0, trying to parse it as a raw FAT32 volume\n", .{});
    }

    const info = parseFat32(&fat32_sector) orelse {
        std.debug.print("no valid FAT32 boot sector found\n", .{});
        return;
    };

    std.debug.print("\nFAT32 boot sector:\n", .{});
    std.debug.print("  bytes_per_sector:    {d}\n", .{info.bytes_per_sector});
    std.debug.print("  sectors_per_cluster: {d}\n", .{info.sectors_per_cluster});
    std.debug.print("  reserved_sectors:    {d}\n", .{info.reserved_sectors});
    std.debug.print("  num_fats:            {d}\n", .{info.num_fats});
    std.debug.print("  fat_size_32:         {d}\n", .{info.fat_size_32});
    std.debug.print("  total_sectors_32:    {d}\n", .{info.total_sectors_32});
    std.debug.print("  root_cluster:        {d}\n", .{info.root_cluster});

    var entries: [MAX_ENTRIES]DirEntry = undefined;
    var count: usize = 0;

    const cluster = info.root_cluster;
    var s: u32 = 0;
    while (s < info.sectors_per_cluster) : (s += 1) {
        const sector_num = clusterToSector(info, cluster) + s;
        try file.seekTo(mounted_offset + @as(u64, sector_num) * SECTOR_SIZE);
        var buf: [SECTOR_SIZE]u8 = undefined;
        _ = try file.readAll(&buf);
        const done = parseDirEntries(&buf, &entries, &count);
        if (done) break;
    }

    std.debug.print("\nroot directory ({d} entries):\n", .{count});
    var name_buf: [12]u8 = undefined;
    var i: usize = 0;
    while (i < count) : (i += 1) {
        const entry = entries[i];
        const name = formatShortName(&name_buf, entry.name);
        const is_dir = entry.attr & 0x10 != 0;
        std.debug.print("  {s}{s}  {d} bytes  cluster={d}\n", .{
            name,
            if (is_dir) " (dir)" else "",
            entry.size,
            entry.cluster,
        });
    }
}
