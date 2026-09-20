#[tokio::test]
async fn test_decode_concurrency_semaphore_limits_parallelism() {
    let semaphore = kglance::features::image::decode_semaphore();
    assert_eq!(semaphore.available_permits(), 2);

    let permit1 = semaphore.try_acquire();
    assert!(permit1.is_ok());
    assert_eq!(semaphore.available_permits(), 1);

    let permit2 = semaphore.try_acquire();
    assert!(permit2.is_ok());
    assert_eq!(semaphore.available_permits(), 0);

    let permit3 = semaphore.try_acquire();
    assert!(permit3.is_err());

    drop(permit1);
    assert_eq!(semaphore.available_permits(), 1);
    drop(permit2);
    assert_eq!(semaphore.available_permits(), 2);
}
