import datetime, pathlib, sys, os, json, threading, queue

WORKDIR = pathlib.Path(__file__).resolve().parents[1]
LOG_DIR = WORKDIR / "logs"
LOG_DIR.mkdir(exist_ok=True)

# Create a queue for logs that can be consumed by the UI
log_queue = queue.Queue(maxsize=1000)  # Limit to prevent memory issues
log_listeners = set()

def log(msg: str, level="INFO"):
    """Log a message to console, file, and make it available to UI"""
    stamp = datetime.datetime.now().strftime("%F %T")
    line = f"[{stamp}] {msg}"
    print(line, flush=True)
    
    # Write to log file
    with open(LOG_DIR / f"agent_{datetime.date.today()}.log", "a") as f:
        f.write(line + "\n")
    
    # Add to queue for UI
    log_entry = {
        "timestamp": stamp,
        "message": msg,
        "level": level
    }
    
    try:
        log_queue.put_nowait(log_entry)
    except queue.Full:
        # If queue is full, remove oldest item
        try:
            log_queue.get_nowait()
            log_queue.put_nowait(log_entry)
        except:
            pass  # If concurrent access issues, just skip this log for the queue
    
    # Notify all listeners
    for callback in log_listeners:
        try:
            callback(log_entry)
        except:
            pass  # Ignore errors in callbacks

def get_recent_logs(limit=100):
    """Get recent logs for UI display"""
    logs = []
    # Copy from queue without removing items
    try:
        q_size = log_queue.qsize()
        for _ in range(min(limit, q_size)):
            item = log_queue.get()
            logs.append(item)
            log_queue.put(item)  # Put it back
    except:
        pass  # Ignore queue access issues
    return logs

def add_log_listener(callback):
    """Add a callback function that will be called for each new log entry"""
    log_listeners.add(callback)
    return callback

def remove_log_listener(callback):
    """Remove a log listener"""
    if callback in log_listeners:
        log_listeners.remove(callback)
