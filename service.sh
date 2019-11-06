#!/bin/bash

source ./environment.deprecated

get_stdout_file() {
    echo -n "/tmp/$1.log"
}
get_stderr_file() {
    echo -n "/tmp/$1.err"
}
get_pid_file() {
    echo -n "/tmp/$1.pid"
}
get_pid() {
    pid_file=`get_pid_file $1`
    cat "$pid_file"
}
is_running() {
    pid_file=`get_pid_file $1`
    [ -f "$pid_file" ] && ps -p `get_pid $1` > /dev/null 2>&1
}

case "$1" in
    start)
    for name in tms tdfs fns kms ; do
        if is_running $name; then
            echo "$name already started";
            exit 1
        fi
    done
    cd $MESATEE_CFG_DIR/bin
    for name in tms tdfs fns kms ; do
        echo "Starting $name"
        stdout_log=`get_stdout_file $name`
        stderr_log=`get_stderr_file $name`
        pid_file=`get_pid_file $name`
        ./$name >> "$stdout_log" 2>> "$stderr_log" &
        echo $! > "$pid_file"
        if ! is_running $name; then
             echo "Unable to start $name, see $stdout_log and $stderr_log"
             exit 1
        fi
    done
    ;;
    stop)
    for name in tms tdfs fns kms ; do
        if is_running $name; then
            echo "Stopping $name.."
            kill `get_pid $name`
        else
            echo "$name not running"
        fi
    done
    for name in tms tdfs fns kms ; do
        for i in 1 2 3 4 5 6 7 8 9 10; do
            if ! is_running $name; then
                break
            fi
            echo -n "."
            sleep 1
        done
        if is_running $name; then
            echo "Not stopped $name; may still be shutting down or shutdown may have failed"
        else
            echo "Stopped $name"
            pid_file=`get_pid_file $name`
            if [ -f "$pid_file" ]; then
                rm "$pid_file"
            fi
        fi
    done
    ;;
    restart)
    bash $0 stop
    for name in tms tdfs fns kms ; do
        if is_running $name; then
            echo "Unable to stop $name, will not attempt to start"
            exit 1
        fi
    done
    bash $0 start
    ;;
    *)
    echo "Usage: $0 {start|stop|restart}"
    exit 1
    ;;
esac

exit 0
