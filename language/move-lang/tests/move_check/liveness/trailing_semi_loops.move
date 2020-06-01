script {
    fun main() {
        loop ();
    }
}

script {
    fun main() {
        { (loop (): ()) };
    }
}

script {
    fun main() {
        loop {
            let x = 0;
            0 + x + 0;
        };
    }
}

script {
    fun main() {
        loop {
            // TODO can probably improve this message,
            // but its different than the normal trailing case
            let x: u64 = if (true) break else break;
        }
    }
}

script {
    fun main() {
        loop {
            break;
        }
    }
}

script {
    fun main(cond: bool) {
        loop {
            if (cond) {
                break;
            } else {
                ()
            }
        }
    }
}

script {
    fun main(cond: bool) {
        loop {
            if (cond) continue else break;
        }
    }
}

script {
    fun main(cond: bool) {
        loop {
            if (cond) abort 0 else return;
        }
    }
}

script {
    fun main(cond: bool) {
        let x;
        loop {
            if (cond) {
                x = 1;
                break
            } else {
                x = 2;
                continue
            };
        };
        x;
    }
}

script {
    fun main(cond: bool) {
        loop {
            if (cond) {
                break;
            } else {
                continue;
            };
        }
    }
}

script {
    fun main(cond: bool) {
        loop {
            if (cond) {
                return;
            } else {
                abort 0;
            };
        }
    }
}
