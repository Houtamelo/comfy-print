### This crate provides non-panicking versions of:
- std::print!()
- std::println!()
- std::eprint!()
- std::eprintln!()

> Print! can panic???

Astonishingly, yes: [issue](https://github.com/rust-lang/rust/issues/24821).

It's very rare, you might never see it, you can also guarantee that you won't see it by using this crate.

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

Here we try to write to the desired output. If that fails, we insert an error message in front of the queue, then the original message afterwards.

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

The async version has a very similar concept to the sync one. The main difference is that, instead of blocking the current one, printing the queue is done from a spawned thread:

```rs
static ACTIVE_THREAD: FairMutex<Option<JoinHandle<()>>> = FairMutex::new(None);
```
Instead of taking the responsibility of printing by setting "IS_PRINTING", we spawn a thread and store it's handle on `ACTIVE_THREAD`.

By having direct access to the handle, we also gain more stability by depending on a direct handle instead of a decoupled boolean.

<details>
    <summary>pub fn _comfy_print_async(msg: Message)</summary>

```rs
pub fn _comfy_print_async(msg: Message) {
    let mut queue_guard = QUEUE.lock();
    
    if queue_guard.len() == 0 {
        drop(queue_guard);
        write_first_in_line(msg);
    } else {
        queue_guard.push(msg);
        drop(queue_guard);
        check_thread();
    }
}
```
Just like in the sync version, we lock the queue and check if it's empty.

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
Then we are clear to try printing the message in the same thread, this is what happens most of the time.

<details>
    <summary>fn write_first_in_line(msg: Message)</summary>

```rs
fn write_first_in_line(msg: Message) {
    let msg_str: &str = msg.str();
    
    if let Err(err) = utils::try_write(&msg_str, msg.output_kind()) {
        let mut queue_guard = QUEUE.lock();
        queue_guard.insert(0, Message::error(format!(
            "comfy_print::blocking_write_first_in_line(): Failed to print first message in queue, it was pushed to the front again.\n\
            Error: {err}\n\
            Message: {msg_str}")));
        
        queue_guard.insert(1, msg);
    }
}
```
Works exactly like the sync version, attempting to lock output, then print the message.

If writing to output fails, we insert the error in the front of the queue, and the original message afterwards.

</details>

</details>

---

<details>
    <summary>When Queue has elements</summary>

```rs
} else {
    queue_guard.push(msg);
    drop(queue_guard);
    check_thread();
}
```

We push the requested message on the queue, then immediately release the lock in case there's a thread waiting for it. 

Finally, we check if there's an active thread printing the queue.

<details>
    <summary>fn check_thread()</summary>

```rs
fn check_thread() {
    let Some(mut thread_guard) = ACTIVE_THREAD.try_lock()
        else { return; };
    
    let is_printing = thread_guard.as_ref().is_some_and(|handle| !handle.is_finished());
    if is_printing { // We already pushed our msg to the queue and there's already a thread printing it, so we can return.
        return;
    }
    
    match thread::Builder::new().spawn(print_until_empty) {
        Ok(ok) => {
            *thread_guard = Some(ok);
            drop(thread_guard);
        },
        Err(err) => { // We couldn't create a thread, we'll have to block this one
            drop(thread_guard);
            
            let mut queue_guard = QUEUE.lock();
            queue_guard.insert(0, Message::error(format!(
                "comfy_print::queue_then_check_thread(): Failed to create a thread to print the queue.\n\
                Error: {err}.")));
            
            drop(queue_guard);
            print_until_empty();
        }
    }
}
```

Once again, there's a lot going on here so let's divide into smaller steps:

```rs
let Some(mut thread_guard) = ACTIVE_THREAD.try_lock()
    else { return; };

let is_printing = thread_guard.as_ref().is_some_and(|handle| !handle.is_finished());
if is_printing { // We already pushed our msg to the queue and there's already a thread printing it, so we can return.
    return;
}
```

First, by trying to acquire the lock, we perform a non-blocking operation that tells us if there's another thread already using it. 
If that's the case, we can assume that the other thread is also about to start printing the queue. We can stop here.

If we did acquire the lock, we can check if there's anything there, and if the handle inside belongs to a thread that's already finished.

If the handle exists and it's not finished, then it means there's another thread printing the queue, so we can also return.

```rs
match thread::Builder::new().spawn(print_until_empty) {
```

Here we try spawning a new thread, requesting that it executes the function `fn print_until_empty()`. 

```rs
Ok(handle) => {
    *thread_guard = Some(handle);
    drop(thread_guard);
},
```

If spawning succeeds, we insert the handle in our static Mutex, other threads will check it to see if they can take the responsibility of printing.

As usual, we also immediately release the lock that we are holding.

```rs
Err(err) => {
    drop(thread_guard);
    
    let mut queue_guard = QUEUE.lock();
    queue_guard.insert(0, Message::error(format!(
        "comfy_print::queue_then_check_thread(): Failed to create a thread to print the queue.\n\
        Error: {err}.")));
    
    drop(queue_guard);
}
```

If, for whatever reason, spawning the thread fails, we have a new error message to print.

As usual, before acquiring a lock on the queue, we release the lock referencing the handle. Then insert the error message in front of the queue.

Once again, we return now and hope that the user calls print again, which would read the queue and attempt to print all the stored messages.

```rs
fn print_until_empty() {
    const MAX_RETRIES: u8 = 50;
    let mut retries = 0;
    
    loop {
        let mut queue_guard = QUEUE.lock();
        
        if queue_guard.len() <= 0 {
            drop(queue_guard);
            break;
        }
        
        let msg = queue_guard.remove(0);
        let msg_str: &str = msg.str();
        let output_kind = msg.output_kind();
        drop(queue_guard); // unlock the queue before blocking stdout/err

        let write_result = utils::try_write(&msg_str, output_kind);
        
        if let Err(err) = write_result {
            let mut queue_guard = QUEUE.lock();
            queue_guard.insert(0, Message::error(format!(
                "comfy_print::write_until_empty(): Failed to print first message in queue.\n\
                Error: {err}\n\
                Message: {msg_str}\n\
                Target output: {output_kind:?}")));
            
            queue_guard.insert(1, msg);
            drop(queue_guard);
            
            retries += 1;
            if retries >= MAX_RETRIES {
                break;
            }
        }

        thread::yield_now();
    }
}
```

This is the function that actually prints the queue, let's break it into smaller steps.

```rs
const MAX_RETRIES: u8 = 50;
let mut retries = 0;
```

For starters, we have an arbitrary integer that defines the maximum number of retries in case a print operation fails, and the local integer `retries` to count the attempts.

```rs
let mut queue_guard = QUEUE.lock();

if queue_guard.len() <= 0 {
    drop(queue_guard);
    break;
}
```

Inside the loop, we lock the queue, then stop if it's empty, as that would mean our job is done.

```rs
let msg = queue_guard.remove(0);
let msg_str: &str = msg.str();
let output_kind = msg.output_kind();
drop(queue_guard); // unlock the queue before blocking stdout/err
```

Just like in the sync version, we pop the front element out of the queue, then immediately release the lock.

Releasing the lock here also ensures we don't hold two locks at once, as we are about to lock the output stream.

```rs
let write_result = utils::try_write(&msg_str, output_kind);

if let Err(err) = write_result {
    let mut queue_guard = QUEUE.lock();
    queue_guard.insert(0, Message::error(format!(
        "comfy_print::write_until_empty(): Failed to print first message in queue.\n\
        Error: {err}\n\
        Message: {msg_str}\n\
        Target output: {output_kind:?}")));
    
    queue_guard.insert(1, msg);
    drop(queue_guard);
    
    retries += 1;
    if retries >= MAX_RETRIES {
        break;
    }
}

thread::yield_now();
```

If writing to output fails, just like in the sync version, we'll insert an error message in front of the queue, then the original message afterwards.

However, since we are guaranteed to not be in the main thread, we can hold the print responsibility for a bit longer, we'll keep trying to print up to `MAX_RETRIES`.

Regardless of the print result, at the end of each iteration we call `thread::yield_now();` this will give other threads a chance to hopefully un-screw the output stream, while also allowing more messages to join the queue.

  </details>
</details>

---

## QA

> Why explicitly call `drop(guard)` on instances where it would automatically be called implicitly in the same order?

A: To take care of my future self: by leaving it implicit, I'm counting on my future brain to read the code and figure out the exact order of guards being locked/unlocked.

By explicitly writing `drop(guard)`, I'm making it clear where locks are released, thus my future brain will have less opportunities to make mistakes.

This also makes it clear for other programmers reading the code, they will have an easier time understanding my intentions.

---

> This is over-engineered

A: Yes, I wrote this crate with the intent of learning/practicing threads/concurrency in Rust. 

I also really hate the idea of seeing a `print!` call panic.

---

> You call the versions sync/async, but both use threads

Both versions are thread safe, as in, they are aware of threads, and their code was written taking into account that `print!` might be called from different threads, simultaneously.

However, only the async version actually spawns a thread to print the queue, the sync version does it in the current thread.
