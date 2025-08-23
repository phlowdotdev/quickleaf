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
    fn test_list_respects_limit() {
        let mut cache = Cache::new(20);
        
        // Insert 15 items
        for i in 0..15 {
            cache.insert(format!("key_{:02}", i), i);
        }

        // Test with limit of 5
        let props = ListProps::default().limit(5);
        let results = cache.list(props).unwrap();
        assert_eq!(results.len(), 5);
        
        // Verify they are the first 5 in ascending order
        assert_eq!(results[0].0, "key_00");
        assert_eq!(results[1].0, "key_01");
        assert_eq!(results[2].0, "key_02");
        assert_eq!(results[3].0, "key_03");
        assert_eq!(results[4].0, "key_04");

        // Test with limit of 0 (should return nothing)
        let props = ListProps::default().limit(0);
        let results = cache.list(props).unwrap();
        assert_eq!(results.len(), 0);

        // Test with limit greater than items (should return all)
        let cache_len = cache.len();
        let props = ListProps::default().limit(20);
        let results = cache.list(props).unwrap();
        assert_eq!(results.len(), cache_len);
    }

    #[test]
    fn test_list_respects_start_after_key() {
        let mut cache = Cache::new(10);
        
        // Insert items with predictable keys
        for i in 0..10 {
            cache.insert(format!("key_{:02}", i), i);
        }

        // Test starting after key_05
        let props = ListProps::default().start_after_key("key_05");
        let results = cache.list(props).unwrap();
        
        // Should return key_06 through key_09 (4 items)
        assert_eq!(results.len(), 4);
        assert_eq!(results[0].0, "key_06");
        assert_eq!(results[1].0, "key_07");
        assert_eq!(results[2].0, "key_08");
        assert_eq!(results[3].0, "key_09");

        // Test starting after last key (should return empty)
        let props = ListProps::default().start_after_key("key_09");
        let results = cache.list(props).unwrap();
        assert_eq!(results.len(), 0);

        // Test starting after non-existent key (should return error)
        let props = ListProps::default().start_after_key("non_existent");
        let results = cache.list(props);
        assert!(results.is_err());
    }

    #[test]
    fn test_list_pagination_combined() {
        let mut cache = Cache::new(20);
        
        // Insert 20 items
        for i in 0..20 {
            cache.insert(format!("key_{:02}", i), i);
        }

        // Get first page: 5 items starting from beginning
        let props = ListProps::default().limit(5);
        let page1 = cache.list(props).unwrap();
        assert_eq!(page1.len(), 5);
        assert_eq!(page1[0].0, "key_00");
        assert_eq!(page1[4].0, "key_04");

        // Get second page: 5 items starting after key_04
        let props = ListProps::default()
            .start_after_key("key_04")
            .limit(5);
        let page2 = cache.list(props).unwrap();
        assert_eq!(page2.len(), 5);
        assert_eq!(page2[0].0, "key_05");
        assert_eq!(page2[4].0, "key_09");

        // Get third page: 5 items starting after key_09
        let props = ListProps::default()
            .start_after_key("key_09")
            .limit(5);
        let page3 = cache.list(props).unwrap();
        assert_eq!(page3.len(), 5);
        assert_eq!(page3[0].0, "key_10");
        assert_eq!(page3[4].0, "key_14");

        // Get fourth page: should return remaining 5 items
        let props = ListProps::default()
            .start_after_key("key_14")
            .limit(5);
        let page4 = cache.list(props).unwrap();
        assert_eq!(page4.len(), 5);
        assert_eq!(page4[0].0, "key_15");
        assert_eq!(page4[4].0, "key_19");

        // Try to get fifth page: should be empty
        let props = ListProps::default()
            .start_after_key("key_19")
            .limit(5);
        let page5 = cache.list(props).unwrap();
        assert_eq!(page5.len(), 0);
    }

    #[test]
    fn test_list_pagination_with_filter() {
        let mut cache = Cache::new(30);
        
        // Insert items with different prefixes
        for i in 0..10 {
            cache.insert(format!("user_{:02}", i), i);
            cache.insert(format!("admin_{:02}", i), i + 10);
            cache.insert(format!("guest_{:02}", i), i + 20);
        }

        // Get first page of user items
        let props = ListProps::default()
            .filter(Filter::StartWith("user_".to_string()))
            .limit(3);
        let page1 = cache.list(props).unwrap();
        assert_eq!(page1.len(), 3);
        assert_eq!(page1[0].0, "user_00");
        assert_eq!(page1[1].0, "user_01");
        assert_eq!(page1[2].0, "user_02");

        // Get second page of user items
        let props = ListProps::default()
            .filter(Filter::StartWith("user_".to_string()))
            .start_after_key("user_02")
            .limit(3);
        let page2 = cache.list(props).unwrap();
        assert_eq!(page2.len(), 3);
        assert_eq!(page2[0].0, "user_03");
        assert_eq!(page2[1].0, "user_04");
        assert_eq!(page2[2].0, "user_05");
    }

    #[test]
    fn test_list_pagination_desc_order() {
        let mut cache = Cache::new(15);
        
        // Insert 15 items
        for i in 0..15 {
            cache.insert(format!("key_{:02}", i), i);
        }

        // Get first page in descending order
        let props = ListProps::default()
            .order(Order::Desc)
            .limit(5);
        let page1 = cache.list(props).unwrap();
        assert_eq!(page1.len(), 5);
        assert_eq!(page1[0].0, "key_14");
        assert_eq!(page1[4].0, "key_10");

        // Get second page in descending order
        let props = ListProps::default()
            .order(Order::Desc)
            .start_after_key("key_10")
            .limit(5);
        let page2 = cache.list(props).unwrap();
        assert_eq!(page2.len(), 5);
        assert_eq!(page2[0].0, "key_09");
        assert_eq!(page2[4].0, "key_05");

        // Get third page in descending order
        let props = ListProps::default()
            .order(Order::Desc)
            .start_after_key("key_05")
            .limit(5);
        let page3 = cache.list(props).unwrap();
        assert_eq!(page3.len(), 5);
        assert_eq!(page3[0].0, "key_04");
        assert_eq!(page3[4].0, "key_00");
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
