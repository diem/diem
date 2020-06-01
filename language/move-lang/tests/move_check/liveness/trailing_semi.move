script {
    fun main() {
        return;
    }
}

script {
    fun main() {
        abort 0;
    }
}

script {
    fun main() {
        { return };
    }
}

script {
    fun main() {
        { abort 0 };
    }
}


script {
    fun main(cond: bool) {
        if (cond) {
            return;
        } else {
            ()
        }
    }
}

script {
    fun main(cond: bool) {
        {
            if (cond) {
                return
            } else {
                abort 0
            };
        }
    }
}

script {
    fun main(cond: bool) {
        {
            if (cond) {
                abort 0;
            } else {
                return;
            };
        }
    }
}
