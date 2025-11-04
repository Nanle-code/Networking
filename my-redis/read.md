The client::connect function is provided by the mini-redis crate. It asynchronously establishes a TCP connection with the specified remote address. 
Once the connection is established, a client handle is returned. Even though the operation is performed asynchronously, the code we write looks synchronous. The only indication that the operation is asynchronous is the .await operator.

Asynchronous programming lets a program continue running other tasks while waiting for slow operations (like network connections) instead of blocking the thread. Unlike synchronous code that pauses until a task finishes, async code suspends and later resumes once the operation completes.
Though faster, it can be complex because developers must manage the program’s state between suspensions.

Rust simplifies this using the async/await feature — allowing asynchronous code to be written in a clear, sequential style, while the compiler manages the underlying state and scheduling (a form of compile-time green-threading).