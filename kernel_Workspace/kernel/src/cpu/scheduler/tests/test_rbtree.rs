use crate::cpu::scheduler::entities::task::{TaskStruct, SchedPolicy};
use crate::cpu::scheduler::runqueue::rbtree;
use core::ptr;

fn fair_key(node: *const TaskStruct) -> u64 {
    unsafe { (*node).se.vruntime }
}

fn dl_key(node: *const TaskStruct) -> u64 {
    unsafe { (*node).dl.deadline }
}

fn make_task(pid: u64, vruntime: u64, deadline: u64) -> TaskStruct {
    let mut t = TaskStruct::blank();
    t.init_test_stub(pid, SchedPolicy::Normal, 0);
    t.se.vruntime = vruntime;
    t.dl.deadline = deadline;
    t
}

#[test]
fn empty_tree_has_zero_count_and_ok_invariants() {
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();
    unsafe {
        assert_eq!(rbtree::count(root), 0);
        assert!(rbtree::check_invariants(root, fair_key).is_ok());
        assert!(leftmost.is_null());
    }
}

#[test]
fn single_node_tree_is_black_and_valid() {
    let mut t = make_task(1, 100, 0);
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();
    unsafe {
        rbtree::insert(&mut root, &mut leftmost, &mut t as *mut _, fair_key);
        assert_eq!(rbtree::count(root), 1);
        assert_eq!(leftmost, &mut t as *mut _);
        assert!(rbtree::check_invariants(root, fair_key).is_ok());
    }
}

#[test]
fn sequential_inserts_maintain_sorted_order_and_invariants() {
    const N: usize = 100;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();

    for i in 0..N {
        tasks[i] = make_task(i as u64, i as u64 * 10, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
            assert!(rbtree::check_invariants(root, fair_key).is_ok());
        }
    }

    unsafe {
        assert_eq!(rbtree::count(root), N);
        let mut prev_vrt = 0;
        let mut curr = leftmost;
        let mut visited = 0;
        while !curr.is_null() {
            let vrt = (*curr).se.vruntime;
            assert!(vrt >= prev_vrt);
            prev_vrt = vrt;
            visited += 1;
            curr = rbtree::successor(curr);
        }
        assert_eq!(visited, N);
    }
}

#[test]
fn reverse_inserts_maintain_invariants() {
    const N: usize = 50;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();

    for i in (0..N).rev() {
        tasks[i] = make_task(i as u64, i as u64 * 100, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    unsafe {
        assert!(rbtree::check_invariants(root, fair_key).is_ok());
        assert_eq!((*leftmost).se.vruntime, 0);
    }
}

#[test]
fn random_inserts_and_deletes_keep_tree_valid() {
    const N: usize = 200;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();
    let mut seed = 0x123456789ABCDEFu64;

    let mut next_u64 = || -> u64 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };

    for i in 0..N {
        tasks[i] = make_task(i as u64, next_u64() % 1_000_000, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    unsafe {
        assert_eq!(rbtree::count(root), N);
        assert!(rbtree::check_invariants(root, fair_key).is_ok());
    }

    for i in (0..N).step_by(2) {
        unsafe {
            rbtree::delete(&mut root, &mut leftmost, &mut tasks[i] as *mut _);
            assert!(rbtree::check_invariants(root, fair_key).is_ok());
        }
    }

    unsafe {
        assert_eq!(rbtree::count(root), N / 2);
    }
}

#[test]
fn delete_root_repeatedly_destroys_tree_cleanly() {
    const N: usize = 30;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();

    for i in 0..N {
        tasks[i] = make_task(i as u64, i as u64, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    unsafe {
        while !root.is_null() {
            let min = leftmost;
            rbtree::delete(&mut root, &mut leftmost, min);
            assert!(rbtree::check_invariants(root, fair_key).is_ok());
        }
        assert!(leftmost.is_null());
    }
}

#[test]
fn delete_successor_maintains_inorder_traversal() {
    const N: usize = 40;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();

    for i in 0..N {
        tasks[i] = make_task(i as u64, (i * 7) % 100, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    unsafe {
        let mut collected_before = Vec::new();
        let mut curr = leftmost;
        while !curr.is_null() {
            collected_before.push((*curr).se.vruntime);
            curr = rbtree::successor(curr);
        }

        let target_vrt = collected_before[N / 2];
        let mut target = ptr::null_mut();
        for i in 0..N {
            if tasks[i].se.vruntime == target_vrt {
                target = &mut tasks[i] as *mut _;
                break;
            }
        }

        rbtree::delete(&mut root, &mut leftmost, target);
        assert!(rbtree::check_invariants(root, fair_key).is_ok());

        let mut collected_after = Vec::new();
        curr = leftmost;
        while !curr.is_null() {
            collected_after.push((*curr).se.vruntime);
            curr = rbtree::successor(curr);
        }

        collected_before.retain(|&x| x != target_vrt);
        assert_eq!(collected_before, collected_after);
    }
}

#[test]
fn duplicate_vruntime_inserts_are_placed_to_the_right() {
    const N: usize = 10;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();

    for i in 0..N {
        tasks[i] = make_task(i as u64, 500, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    unsafe {
        assert_eq!(rbtree::count(root), N);
        assert!(rbtree::check_invariants(root, fair_key).is_ok());
        let mut count = 0;
        let mut curr = leftmost;
        while !curr.is_null() {
            assert_eq!((*curr).se.vruntime, 500);
            count += 1;
            curr = rbtree::successor(curr);
        }
        assert_eq!(count, N);
    }
}

#[test]
fn deadline_key_orders_by_earliest_deadline() {
    let mut t1 = make_task(1, 0, 1000);
    let mut t2 = make_task(2, 0, 500);
    let mut t3 = make_task(3, 0, 2000);
    
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();

    unsafe {
        rbtree::insert(&mut root, &mut leftmost, &mut t1 as *mut _, dl_key);
        rbtree::insert(&mut root, &mut leftmost, &mut t2 as *mut _, dl_key);
        rbtree::insert(&mut root, &mut leftmost, &mut t3 as *mut _, dl_key);

        assert_eq!((*leftmost).dl.deadline, 500);
        assert_eq!((*rbtree::successor(leftmost)).dl.deadline, 1000);
        assert!(rbtree::check_invariants(root, dl_key).is_ok());
    }
}

#[test]
fn subtree_min_and_max_return_extremes() {
    let mut tasks: [TaskStruct; 5] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();
    let values = [50, 10, 90, 30, 70];

    for (i, &v) in values.iter().enumerate() {
        tasks[i] = make_task(i as u64, v, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    unsafe {
        let min = rbtree::subtree_min(root);
        let max = rbtree::subtree_max(root);
        assert_eq!((*min).se.vruntime, 10);
        assert_eq!((*max).se.vruntime, 90);
    }
}

#[test]
fn stress_test_insert_delete_cycles() {
    const N: usize = 500;
    let mut tasks: [TaskStruct; N] = core::array::from_fn(|_| TaskStruct::blank());
    let mut root = ptr::null_mut();
    let mut leftmost = ptr::null_mut();
    let mut seed = 0xDEADBEEFCAFEBABEu64;

    let mut next_u64 = || -> u64 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };

    for i in 0..N {
        tasks[i] = make_task(i as u64, next_u64() % 10_000, 0);
        unsafe {
            rbtree::insert(&mut root, &mut leftmost, &mut tasks[i] as *mut _, fair_key);
        }
    }

    for cycle in 0..5 {
        for i in (cycle..N).step_by(3) {
            unsafe {
                if tasks[i].se.on_rq {
                    rbtree::delete(&mut root, &mut leftmost, &mut tasks[i] as *mut _);
                }
            }
        }
        unsafe {
            assert!(rbtree::check_invariants(root, fair_key).is_ok());
        }
    }
}