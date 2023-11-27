#[macro_export]
macro_rules! ok_or_continue{
    ($exec: expr, $content: expr)=>{
        match $exec {
            Ok(value) => value,
            Err(e) => {
                error!("{} failed: {}", $content, e);
                continue;
            }
        }
    };
    ($exec: expr, $content: expr, $operation: expr)=>{
      match $exec {
          Ok(value) => value,
          Err(e) => {
              error!("{} failed: {}", $content, e);
              $operation;
          }
      }
    };
}
