#!/bin/bash

# Fix imports in documentation examples
find src/ -name "*.rs" -exec sed -i \
  -e 's/use quickleaf::cache::Cache;/use quickleaf::Cache;/g' \
  -e 's/use quickleaf::cache::CacheItem;/use quickleaf::CacheItem;/g' \
  -e 's/use quickleaf::error::Error;/use quickleaf::Error;/g' \
  -e 's/use quickleaf::event::Event;/use quickleaf::Event;/g' \
  -e 's/use quickleaf::event::EventData;/use quickleaf::EventData;/g' \
  -e 's/use quickleaf::filter::Filter;/use quickleaf::Filter;/g' \
  -e 's/use quickleaf::list_props::ListProps;/use quickleaf::ListProps;/g' \
  -e 's/use quickleaf::list_props::Order;/use quickleaf::Order;/g' \
  -e 's/use quickleaf::list_props::StartAfter;/use quickleaf::StartAfter;/g' \
  -e 's/use quickleaf::{ListProps, Order};/use quickleaf::{ListProps, Order};/g' \
  -e 's/use quickleaf::{StartAfter, ListProps, Order};/use quickleaf::{StartAfter, ListProps, Order};/g' \
  {} \;

echo "Fixed documentation imports"
