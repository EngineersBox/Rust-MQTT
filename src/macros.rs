#[macro_export]
macro_rules! ternary {
    ($condition:expr, $v1:expr, $v2:expr) => {
        if $condition { $v1 } else { $v2 };
    };
}

#[macro_export]
macro_rules! if_let_some {
    ($var:pat = $value:expr) => {
        let $var = if let Some(it) = $value {
            it
        } else {
            return;
        };
    };

    ($var:pat = $value:expr, $else_value:expr) => {
        #[allow(clippy::if_let_some_result)]
        let $var = if let Some(it) = $value {
            it
        } else {
            return $else_value;
        };
    }
}

#[macro_export]
macro_rules! if_let_ok {
    ($var:pat = $value:expr, $else_value:expr) => {
        let $var = match $value {
            Ok(it) => it,
            Err(err) => return $else_value(err),
        };
    }
}

#[macro_export]
macro_rules! try_except_return {
    ($connection_statement:expr, $msg:literal, $logger:expr) => {
        match $connection_statement {
            Ok(value) => value,
            Err(e) => {
                error!($logger, "{}: {}", $msg, e);
                return;
            },
        }
    }
}

#[macro_export]
macro_rules! try_except_return_default {
    ($connection_statement:expr, $msg:literal, $default_value:expr, $logger:expr) => {
        match $connection_statement {
            Ok(value) => value,
            Err(e) => {
                error!($logger, "{}: {}", $msg, e);
                $default_value
            },
        }
    }
}

#[macro_export]
macro_rules! option_same_block {
    ($conditional:expr, $some_statement:expr) => {
        if $conditional {
            return Option::Some($some_statement);
        }
        return Option::None;
    }
}

#[macro_export]
macro_rules! get_current_thread_id {
    () => {
        o!("thread-id" => format!("{:?}", thread::current().id()))
    }
}

#[macro_export]
macro_rules! join_threads {
    ($threads:expr, $thread_logger:expr) => {
        for t in $threads {
            match t.join() {
                Ok(_) => {},
                Err(e) => {
                    crit!($thread_logger, "Handler thread panicked while joining");
                    panic!("Join panic reason: {:?}", e);
                }
            }
        }
    }
}

#[macro_export]
macro_rules! try_except_with_log_action {
    ($matcher:expr, $level:expr, $msg:literal, $t_tx:expr, $log_to:expr) => {
        match $matcher {
            Ok(value) => {
                $log_to.log_at(Level::Debug, "Sent thread message");
                value
            },
            Err(e) => {
                $log_to.log_at($level,  format!("{}: {}", $msg, e).as_str());
                drop($t_tx.clone());
                return;
            },
        }
    }
}
#[macro_export]
macro_rules! message_range_check {
    ($range:expr, $target_value:expr, $target_name:expr, $current:expr, $log_to:expr) => {
        if $range.contains(&$target_value) {
            $current = $target_value;
            if $target_name == "QoS" {
                continue;
            }
        } else if $target_value != -1 {
            $log_to.log_at(Level::Error, format!("{} was not within range {:?}: {}", $target_name, $range, $target_value).as_str());
            continue;
        }
    }
}