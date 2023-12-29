### This crate provides non-panicking versions of:
- std::print!()
- std::println!()
- std::eprint!()
- std::eprintln!()

---

# Usage

## Step 1 - Dependency
Add `comfy-print` as a dependency to your project.

## Step 2 - Replace macro invocations
- Replace every invocation of `std::print!()`    with `comfy_print::print!`
- Replace every invocation of `std::println!()`  with `comfy_print::println!`
- Replace every invocation of `std::eprint!()`   with `comfy_print::eprint!`
- Replace every invocation of `std::eprintln!()` with `comfy_print::eprintln!`

If you're familiar with Regex, you can use the "Replace in files" command of your IDE to do it all at once. 

The default shortcut is often `Ctrl + Shift + R` or `Ctrl + Shift + H`.

Here's the patterns that I use (with Jetbrains Intellij IDEs, Java's Regex):
- Match: `(?<!comfy_e?)(?<type>print!|println!|eprint!|eprintln!)`
- Replace: `comfy_print::comfy_${type}`

## Step 3 - Get comfortable

---

# Features
This crate provides two different implementations of each macro, a synchronous and a asynchronous version.

- The default is the synchronous version.
- The asynchronous version is gated behind the feature `async_impl`.
- You may not use both versions at the same time, activating the `async_impl` feature will hide the synchronous version from compiling.
- Code logic explanation about each version is at the end of this ReadMe.

---

# Pros

- No unsafe code.
- Still thread-safe.
- To avoid deadlocks, locks on std(out/err) are acquired/released in quick succession.
- You may still freely use the std print macros, they do not conflict with any of the comfy_print macros.
- Resilient:
    - The provided macros won't simply "silently fail" when writing to std(out/err) returns an error. Instead, this crate keeps the requested prints in a thread-safe queue, then tries to print again later.
    - The queue ensures prints will always be delivered in the order they were requested.

---

# Cons
- Worse performance:
    - Some very basic synthetic benchmarks showed that, on average, the sync version is 20% slower than std::prints, while the async has similar performance. 
    - Both versions have higher performance variance compared to std::prints.

---

# Code logic

---

Both versions use the same data type for storing messages:

<details>
  <summary>File: utils.rs</summary>
  
```rs
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone)]
pub enum OutputKind {
    Stdout,
    Stderr,
}

pub struct Message {
    string: String,
    output: OutputKind,
}

impl Message {
    pub fn str(&self) -> &str {
        return self.string.as_str();
    }
    
    pub fn output_kind(&self) -> OutputKind {
        return self.output;
    }
    
    pub fn standard(string: String) -> Self {
        return Self {
            string,
            output: OutputKind::Stdout,
        };
    }
    
    pub fn error(string: String) -> Self {
        return Self {
            string,
            output: OutputKind::Stderr,
        };
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.string);
    }
}

pub fn try_write(msg_str: &impl Display, output_kind: OutputKind) -> std::io::Result<()> {
    match output_kind {
        OutputKind::Stdout => {
            let mut stdout = std::io::stdout().lock();
            write!(stdout, "{}", msg_str)?;
            stdout.flush()?;
            Ok(())
        }
        OutputKind::Stderr => {
            let mut stderr = std::io::stderr().lock();
            write!(stderr, "{}", msg_str)?;
            stderr.flush()?;
            Ok(())
        }
    }
}

```
</details>

A simple struct `Message` that contains the `string` to print and where it should be printed `std(out/err)`:
- `get()` functions to access the private fields, and a 
- A constructor for each print target.
- To facilitate formatting, I implemented the trait `Display` for `Message`.

Both implementations(sync/async), use this same type for storing the messages to print.

There's also `pub fn try_write(msg_str: &impl Display, output_kind: OutputKind)`, a shared function that both implementations use. It attempts to print the messages to the desired outputs, returning errors if anything fails.

---

Both implementations define the macros for the end user. These macros serve as bridges to our actual code, just like `std::prints` do:

<details>
  <summary>In file: sync_impl.rs</summary>
  
```rs
pub fn _println(mut input: String) {
    input.push('\n');
    _comfy_print_sync(Message::standard(input));
}

pub fn _print(input: String) {
    _comfy_print_sync(Message::standard(input));
}

pub fn _eprint(input: String) {
    _comfy_print_sync(Message::error(input));
}

pub fn _eprintln(mut input: String) {
    input.push('\n');
    _comfy_print_sync(Message::error(input));
}

#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::sync_impl::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::sync_impl::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::sync_impl::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
    ($($arg:tt)*) => {{
        $crate::sync_impl::_eprint(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprintln {
    () => {
        $crate::sync_impl::_eprintln("\n")
    };
    ($($arg:tt)*) => {{
        $crate::sync_impl::_eprintln(std::format!($($arg)*));
    }};
}
```
</details>


<details>
  <summary>In file: async_impl.rs</summary>
  
```rs
pub fn _print(input: String) {
    _comfy_print_async(Message::standard(input));
}

pub fn _println(mut input: String) {
    input.push('\n');
    _comfy_print_async(Message::standard(input));
}

pub fn _eprint(input: String) {
    _comfy_print_async(Message::error(input));
}

pub fn _eprintln(mut input: String) {
    input.push('\n');
    _comfy_print_async(Message::error(input));
}

#[macro_export]
macro_rules! comfy_print {
    ($($arg:tt)*) => {{
        $crate::async_impl::_print(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_println {
    () => {
        $crate::async_impl::_println("\n")
    };
    ($($arg:tt)*) => {{
        $crate::async_impl::_println(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprint {
    ($($arg:tt)*) => {{
        $crate::async_impl::_eprint(std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! comfy_eprintln {
    () => {
        $crate::async_impl::_eprintln("\n")
    };
    ($($arg:tt)*) => {{
        $crate::async_impl::_eprintln(std::format!($($arg)*));
    }};
}
```
</details>

Both versions use the exact same collection type for storing the queued prints:

```rs
use parking_lot::FairMutex;
static QUEUE: FairMutex<Vec<Message>> = FairMutex::new(Vec::new());
```

The queue is represented by a regular `Vec<Message>` wrapped in [parking_lot](https://crates.io/crates/parking_lot)'s `FairMutex`.

A regular `Mutex`, upon being unlocked, will give the lock to the thread that is executed afterwards, regardless of which thread requested the lock first. This type of lock is problematic for our use case, we want to ensure that prints are done in the exact order they were requested. If there are multiple threads waiting for the lock, the regular `Mutex` won't care about which one asked first, which may result in prints being enqueued on the wrong order.

On the other hand, a `FairMutex` makes threads form a queue upon requesting the lock, ensuring that the thread that asked first gets it's turn first.

---

## Sync Version
In the end, all 4 macros end up calling `comfy_print::sync_impl::_comfy_print_sync(msg)`.

Before trying to print, we need to check if there are already other prints in the queue. If there are, we can't print `msg` right away because that would break the ordering of "prints requested -> prints delivered".

Prints will join the queue if they fail to write to their target output.

<details>
  <summary>fn _comfy_print_sync(msg: Message)</summary>

```rs
pub fn _comfy_print_sync(msg: Message) {
    let mut queue_guard = QUEUE.lock();
    
    if queue_guard.len() == 0 {
        drop(queue_guard); // release the queue's lock before locking std(out/err)
        write_first_in_line(msg);
    } else {
        queue_guard.push(msg);
        drop(queue_guard); // release the queue's lock before locking IS_PRINTING
        if let Ok(_) = IS_PRINTING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed) {
            write_until_empty();
        }
    }
}
```
</details>

---

<details>
  <summary>When QUEUE is empty</summary>

```rs
if queue_guard.len() == 0 {
    drop(queue_guard);
    write_first_in_line(msg);
}
```

We don't have to wait for other threads, just try to print right away. This is what happens in most cases.
  
Since we don't need the queue anymore, we immediately release it. Never owning two locks at the same time helps avoiding some deadlocking cases.

  <details>
    <summary>fn write_first_in_line(msg: Message)</summary>

```rs
fn write_first_in_line(msg: Message) {
    let msg_str: &str = msg.str();
    
    if let Err(err) = try_write(&msg_str, msg.output_kind()) {
        let mut queue_guard = QUEUE.lock();
        queue_guard.insert(0, Message::error(
        format!("comfy_print::blocking_write_first_in_line(): Failed to print first message in queue, it was pushed to the front again.\n\
        Error: {err}\n\
        Message: {msg_str}")));
        queue_guard.insert(1, msg);
        drop(queue_guard);
    }
}
```

Here we try to write to the desired output. If that fails, we insert an error message at the front of the queue, and the original message afterwards.

Trying again is unlikely to yield any results, so we shouldn't do anything else. 

We'll try again next time `comfy_print!` is called.
    
  </details>
</details>

---

<details>
  <summary>When QUEUE has elements</summary>
  There may be another thread already printing the queue, we keep track of that using the static atomic bool: `IS_PRINTING`:

```rs
use std::sync::atomic::{AtomicBool, Ordering};
static IS_PRINTING: AtomicBool = AtomicBool::new(false);
```

```rs
} else {
    queue_guard.push(msg);
    
    if let Ok(_) = IS_PRINTING.compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed) {
        write_until_empty(queue_guard);
    }
}
```

  We join the queue, then check if there is already a thread printing it. 
  
  If there isn't, we'll take that responsibility here. 
  
  The method `compare_exchange` checks if `IS_PRINTING == false`: 
  - If yes:
        Sets `IS_PRINTING = true` and returns Ok().
        This means that we signaled other threads that we are printing the queue.
  - Otherwise:
        Just returns an error.
        We don't have to do anything else, as this means that another thread is already printing, and we already pushed our `msg` to the queue.

  For further learning, I recommend checking out Rust's [Atomic types](https://doc.rust-lang.org/std/sync/atomic/index.html) documentation.

  <details>
    <summary>fn write_until_empty()</summary>

```rs
fn write_until_empty() {
    loop {
        let mut queue_guard = QUEUE.lock();
        
        if queue_guard.len() == 0 {
            drop(queue_guard);
            break;
        }
        
        let msg = queue_guard.remove(0);
        drop(queue_guard);
        let msg_str: &str = msg.str();
        let output_kind = msg.output_kind();
        
        if let Err(err) = try_write(&msg_str, output_kind) {
            let mut queue_guard = QUEUE.lock();
            queue_guard.insert(0, Message::error(format!(
            "comfy_print::write_until_empty(): Failed to print first message in queue, it was pushed to the front again.\n\
            Error: {err}\n\
            Message: {msg_str}\n\
            Target output: {output_kind:?}")));
            
            queue_guard.insert(1, msg);
            drop(queue_guard);
            break;
        }
    }
    
    IS_PRINTING.store(false, Ordering::Relaxed); // signal other threads that we are no longer printing.
}
```

There's quite a bit going on here, let's go step by step.

At the beginning of the loop:

```rs
let mut queue_guard = QUEUE.lock();

if queue_guard.len() == 0 {
    drop(queue_guard);
    break;
}
```
We acquire a lock on the queue, then check if it's empty. If it is, we don't have to do anything else, just release the lock and break out of the loop.

```rs
let msg = queue_guard.remove(0);
drop(queue_guard);
let msg_str: &str = msg.str();
let output_kind = msg.output_kind();
```
We pop the first element out of the queue, then immediately release the lock. Other threads might be waiting on the lock and we don't need it anymore.

```rs
let write_result = try_write(&msg_str, output_kind);

if let Err(err) = write_result {
    let mut queue_guard = QUEUE.lock();
    queue_guard.insert(0, Message::error(format!(
        "comfy_print::write_until_empty(): Failed to print first message in queue, it was pushed to the front again.\n\
        Error: {err}\n\
        Message: {msg_str}\n\
        Target output: {output_kind:?}")));
    
    queue_guard.insert(1, msg);
    drop(queue_guard);
    break;
}
```

Here we try to write to the desired output. If any error did happen, it means we failed to print our message.

To alert the user of the error, we lock the queue again, insert that error in front of the queue, then the original message afterwards.

```rs
IS_PRINTING.store(false, Ordering::Relaxed);
```
At the end of `write_until_empty()`, we set `IS_PRITING` to false, signaling other threads that we are no longer holding that responsibility.

  </details>
</details>

---

## Async Version



---
