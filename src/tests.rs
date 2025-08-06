#[cfg(test)]
mod test {
    use valu3::traits::ToValueBehavior;

    use crate::list_props::{Order, StartAfter};
    use crate::{Cache, Event, EventData, Filter, ListProps};

    #[test]
    fn test_cache_insert() {
        let mut cache = Cache::new(2);
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.insert("key3", 3);
        assert_eq!(cache.get("key1"), None);
        assert_eq!(cache.get("key2"), Some(&2.to_value()));
        assert_eq!(cache.get("key3"), Some(&3.to_value().to_value()));
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = Cache::new(2);
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.remove("key1").expect("Error removing key");
        assert_eq!(cache.get("key1"), None);
        cache.insert("key3", 3);
        assert_eq!(cache.get("key3"), Some(&3.to_value().to_value()));
        assert_eq!(cache.get("key2"), Some(&2.to_value()));
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = Cache::new(2);
        cache.insert("key1", 1);
        cache.insert("key2", 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_list_asc() {
        let mut cache = Cache::new(5);
        cache.insert("key2", 2);
        cache.insert("key1", 1);
        cache.insert("key5", 5);
        cache.insert("key4", 4);
        cache.insert("key3", 3);

        let result_res = cache.list(ListProps {
            order: Order::Asc,
            filter: Filter::None,
            start_after_key: StartAfter::Key("key2".to_string()),
            limit: 10,
        });

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("key3".to_string(), &3.to_value()));
        assert_eq!(result[1], ("key4".to_string(), &4.to_value()));
        assert_eq!(result[2], ("key5".to_string(), &5.to_value()));
    }

    #[test]
    fn test_cache_list_asc_with_filter() {
        let mut cache = Cache::new(10);
        cache.insert("postmodern", 8);
        cache.insert("postpone", 6);
        cache.insert("precept", 2);
        cache.insert("postmortem", 9);
        cache.insert("precaution", 3);
        cache.insert("precede", 1);
        cache.insert("precognition", 5);
        cache.insert("postmark", 10);
        cache.insert("postgraduate", 7);
        cache.insert("preconceive", 4);

        let result_res = cache.list(ListProps {
            order: Order::Asc,
            filter: Filter::StartWith("post".to_string()),
            start_after_key: StartAfter::Key("postmodern".to_string()),
            limit: 10,
        });

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("postmortem".to_string(), &9.to_value()));
        assert_eq!(result[1], ("postpone".to_string(), &6.to_value()));
    }

    #[test]
    fn test_cache_list_desc() {
        let mut cache = Cache::new(5);
        cache.insert("key5", 5);
        cache.insert("key1", 1);
        cache.insert("key3", 3);
        cache.insert("key4", 4);
        cache.insert("key2", 2);

        let result_res = cache.list(ListProps {
            order: Order::Desc,
            filter: Filter::None,
            start_after_key: StartAfter::Key("key3".to_string()),
            limit: 10,
        });

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("key2".to_string(), &2.to_value()));
        assert_eq!(result[1], ("key1".to_string(), &1.to_value()));
    }

    #[test]
    fn test_cache_list_desc_with_filter() {
        let mut cache = Cache::new(10);
        cache.insert("postmodern", 8);
        cache.insert("postpone", 6);
        cache.insert("precept", 2);
        cache.insert("postmortem", 9);
        cache.insert("precaution", 3);
        cache.insert("precede", 1);
        cache.insert("precognition", 5);
        cache.insert("postmark", 10);
        cache.insert("postgraduate", 7);
        cache.insert("preconceive", 4);

        let list_props = ListProps::default()
            .order(Order::Desc)
            .filter(Filter::StartWith("post".to_string()))
            .start_after_key("postmodern");

        let result_res = cache.list(list_props);

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("postmark".to_string(), &10.to_value()));
        assert_eq!(result[1], ("postgraduate".to_string(), &7.to_value()));
    }

    #[test]
    fn test_filter_start_with() {
        let mut cache = Cache::new(10);

        cache.insert("postmodern", 8);
        cache.insert("postpone", 6);
        cache.insert("precept", 2);
        cache.insert("postmortem", 9);
        cache.insert("precaution", 3);
        cache.insert("precede", 1);
        cache.insert("precognition", 5);
        cache.insert("postmark", 10);
        cache.insert("postgraduate", 7);
        cache.insert("preconceive", 4);

        let result_res = cache.list(Filter::StartWith("postm".to_string()));

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 3);
        assert_eq!(result[0], ("postmark".to_string(), &10.to_value()));
        assert_eq!(result[1], ("postmodern".to_string(), &8.to_value()));
        assert_eq!(result[2], ("postmortem".to_string(), &9.to_value()));
    }

    #[test]
    fn test_filter_ends_with() {
        let mut cache = Cache::new(10);

        cache.insert("postmodern", 8);
        cache.insert("postpone", 6);
        cache.insert("precept", 2);
        cache.insert("postmortem", 9);
        cache.insert("precaution", 3);
        cache.insert("precede", 1);
        cache.insert("precognition", 5);
        cache.insert("postmark", 10);
        cache.insert("postgraduate", 7);
        cache.insert("preconceive", 4);

        let result_res = cache.list(Filter::EndWith("tion".to_string()));

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], ("precaution".to_string(), &3.to_value()));
        assert_eq!(result[1], ("precognition".to_string(), &5.to_value()));
    }

    #[test]
    fn test_filter_start_and_end_with() {
        let mut cache = Cache::new(10);

        cache.insert("applemorepie", 1);
        cache.insert("banana", 2);
        cache.insert("pineapplepie", 3);

        let list_props = ListProps::default()
            .filter(Filter::StartAndEndWith(
                "apple".to_string(),
                "pie".to_string(),
            ))
            .order(Order::Asc);

        let result_res = cache.list(list_props);

        assert_eq!(result_res.is_ok(), true);

        let result = match result_res {
            Ok(result) => result,
            Err(_) => panic!("Error"),
        };

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], ("applemorepie".to_string(), &1.to_value()));
    }

    #[test]
    fn test_with_sender() {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut cache = Cache::with_sender(10, tx);

        let mut clone_cache = cache.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(1));
            clone_cache.insert("key1", 1);
        });

        cache.insert("key2", 2);
        cache.insert("key3", 3);

        let mut items = Vec::new();

        for data in rx {
            items.push(data);

            if items.len() == 3 {
                break;
            }
        }

        assert_eq!(items.len(), 3);
        assert_eq!(
            items[0],
            Event::Insert(EventData {
                key: "key2".to_string(),
                value: 2.to_value()
            })
        );
        assert_eq!(
            items[1],
            Event::Insert(EventData {
                key: "key3".to_string(),
                value: 3.to_value()
            })
        );
        assert_eq!(
            items[2],
            Event::Insert(EventData {
                key: "key1".to_string(),
                value: 1.to_value()
            })
        );
    }
}
