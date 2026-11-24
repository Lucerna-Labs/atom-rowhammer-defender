// SPDX-License-Identifier: GPL-2.0-only
// CO-RE type fragments. Only static capabilities and open-time metadata are used.
typedef unsigned int u32;
typedef unsigned long long u64;
#define SEC(name) __attribute__((section(name), used))
#define CORE __attribute__((preserve_access_index))
#define U32_FIELD(name, value) int (*name)[value]

struct kobject { const char *name; struct kobject *parent; } CORE;
struct device { struct device *parent; struct kobject kobj; } CORE;
struct input_dev { unsigned long keybit[1]; struct device dev; } CORE;
struct input_handle { struct input_dev *dev; } CORE;
struct evdev { struct input_handle handle; struct device dev; } CORE;
struct evdev_client { struct evdev *evdev; } CORE;
struct super_block { u32 s_dev; } CORE;
struct inode { unsigned long i_ino; struct super_block *i_sb; } CORE;
struct qstr { const unsigned char *name; } CORE;
struct dentry { struct qstr d_name; } CORE;
struct path { struct dentry *dentry; } CORE;
struct file { u32 f_mode; void *private_data; struct inode *f_inode; struct path f_path; } CORE;
struct mm_struct { struct file *exe_file; } CORE;
struct task_struct { struct mm_struct *mm; } CORE;

struct {
    U32_FIELD(type, 27); /* BPF_MAP_TYPE_RINGBUF */
    U32_FIELD(max_entries, 262144);
} events SEC(".maps");
struct {
    U32_FIELD(type, 2); /* BPF_MAP_TYPE_ARRAY */
    U32_FIELD(max_entries, 1);
    u32 *key;
    u64 *value;
} lost SEC(".maps");

static void *(*lookup)(void *, const void *) = (void *)1;
static u64 (*pid_tgid)(void) = (void *)14;
static u64 (*uid_gid)(void) = (void *)15;
static long (*read_kernel)(void *, u32, const void *) = (void *)113;
static long (*read_string)(void *, u32, const void *) = (void *)115;
static u64 (*boot_ns)(void) = (void *)125;
static long (*emit)(void *, const void *, u64, u64) = (void *)130;
static struct task_struct *(*current_task)(void) = (void *)158;
#define READ(field) ({ typeof(field) value = 0; read_kernel(&value, sizeof(value), __builtin_preserve_access_index(&(field))); value; })

struct observation {
    u64 time_ns, ino;
    u32 pid, uid, exe_dev;
    char device[24], executable[64];
    u32 reserved;
};

// Accept the physical sysfs device tree, excluding /sys/devices/virtual.
// Check ancestry at open time so hot-unplug and minor reuse cannot relabel it.
static __attribute__((always_inline)) inline int physical(struct kobject *node) {
    int child_virtual = 0;
    for (int i = 0; i < 40 && node; i++) {
        char name[8] = {};
        if (read_string(name, sizeof(name), READ(node->name)) < 0)
            return 0;
        struct kobject *parent = READ(node->parent);
        if (!parent)
            return !child_virtual && __builtin_memcmp(name, "devices", 8) == 0;
        child_virtual = __builtin_memcmp(name, "virtual", 8) == 0;
        node = parent;
    }
    return 0;
}

// fexit receives the two function arguments and its return value.
// FMODE_READ is the read-capable file mode. A denied open never qualifies.
SEC("fexit/evdev_open")
int keyboard_open(u64 *ctx) {
    if (ctx[2] != 0)
        return 0;
    struct file *file = (void *)ctx[1];
    if (!(READ(file->f_mode) & 1))
        return 0;
    struct evdev_client *client = READ(file->private_data);
    struct evdev *evdev = READ(client->evdev);
    struct input_dev *input = READ(evdev->handle.dev);
    // Same full-keyboard capability criterion as systemd input_id. Linux
    // input devices without a parent (e.g. uinput) are software devices.
    if (!READ(input->dev.parent) || (READ(input->keybit[0]) & 0xfffffffeUL) != 0xfffffffeUL)
        return 0;
    if (!physical(__builtin_preserve_access_index(&input->dev.kobj)))
        return 0;
    struct observation e = {0};
    e.time_ns = boot_ns();
    e.pid = pid_tgid() >> 32;
    e.uid = (u32)uid_gid();
    struct mm_struct *mm = READ(current_task()->mm);
    struct file *exe = READ(mm->exe_file);
    struct inode *ino = READ(exe->f_inode);
    struct super_block *sb = READ(ino->i_sb);
    struct dentry *dentry = READ(exe->f_path.dentry);
    e.ino = READ(ino->i_ino);
    e.exe_dev = READ(sb->s_dev);
    read_string(e.executable, sizeof(e.executable), READ(dentry->d_name.name));
    read_string(e.device, sizeof(e.device), READ(evdev->dev.kobj.name));
    if (emit(&events, &e, sizeof(e), 0) < 0) {
        u32 zero = 0;
        u64 *count = lookup(&lost, &zero);
        if (count)
            __sync_fetch_and_add(count, 1);
    }
    return 0;
}
char LICENSE[] SEC("license") = "GPL";
