---
id: grpc
title: GRPC
custom_edit_url: https://github.com/libra/libra/edit/master/grpc/README.md
---
# Admission Control

GRPC is an umbrella for collecting protobuf definitions and associated tooling.

## Overview
The GRPC package is intended to protect the rest of the codebase from protobuf.  This package should contain all
protobuf definitions, all functions which convert to native rust types, and external facing (g)rpc service associated
code.

## Implementation Details

## How is this module organized?
```
    .
    ├── README.md
    ├── admission_control/     # (planned) admission-control associatd protobuf types and logic
    │
    ├── types                  # Protobuf definition and conversion for libra/types/
    │   └── src
    │       ├── converters/    # Convert from protobuf to rust types
    │       ├── proto/         # Protobuf type definitions
    │       └── unit_tests/    # Conversion tests
    └── storage                # (planned) storage associated protobuf types and logic
```

## This module interacts with:
The `types` component for type definitions
(planned) The `admission_control` component for type definitions
(planned) The `storage` component for type definitions.
