// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#pragma once

#include <array>
#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <stdint.h>
#include <string>
#include <tuple>
#include <type_traits>
#include <variant>
#include <vector>

using uint128_t = std::tuple<uint64_t, uint64_t>;
using int128_t = std::tuple<int64_t, uint64_t>;

/* -------- */

template <typename T>
struct Serializable {
    template <typename Serializer>
    static void serialize(const T &value, Serializer &serializer);
};

template <typename T>
struct Deserializable {
    template <typename Deserializer>
    static T deserialize(Deserializer &deserializer);
};

/* -------- */

template <typename T, typename Deleter>
struct Serializable<std::unique_ptr<T, Deleter>> {
    template <typename Serializer>
    static void serialize(const std::unique_ptr<T, Deleter> &value,
                          Serializer &serializer) {
        Serializable<T>::serialize(*value, serializer);
    }
};

template <typename T, typename Allocator>
struct Serializable<std::vector<T, Allocator>> {
    template <typename Serializer>
    static void serialize(const std::vector<T, Allocator> &value,
                          Serializer &serializer) {
        serializer.serialize_len(value.size());
        for (const T &item : value) {
            Serializable<T>::serialize(item, serializer);
        }
    }
};

template <typename T, std::size_t N>
struct Serializable<std::array<T, N>> {
    template <typename Serializer>
    static void serialize(const std::array<T, N> &value,
                          Serializer &serializer) {
        for (const T &item : value) {
            Serializable<T>::serialize(item, serializer);
        }
    }
};

template <>
struct Serializable<std::string> {
    template <typename Serializer>
    static void serialize(const std::string &value, Serializer &serializer) {
        serializer.serialize_str(value);
    }
};

template <>
struct Serializable<uint8_t> {
    template <typename Serializer>
    static void serialize(const uint8_t &value, Serializer &serializer) {
        serializer.serialize_u8(value);
    }
};

template <>
struct Serializable<uint16_t> {
    template <typename Serializer>
    static void serialize(const uint32_t &value, Serializer &serializer) {
        serializer.serialize_u32(value);
    }
};

template <>
struct Serializable<uint32_t> {
    template <typename Serializer>
    static void serialize(const uint32_t &value, Serializer &serializer) {
        serializer.serialize_u32(value);
    }
};

template <>
struct Serializable<uint64_t> {
    template <typename Serializer>
    static void serialize(const uint64_t &value, Serializer &serializer) {
        serializer.serialize_u64(value);
    }
};

template <>
struct Serializable<uint128_t> {
    template <typename Serializer>
    static void serialize(const uint128_t &value, Serializer &serializer) {
        serializer.serialize_u128(value);
    }
};

template <>
struct Serializable<int8_t> {
    template <typename Serializer>
    static void serialize(const int8_t &value, Serializer &serializer) {
        serializer.serialize_i8(value);
    }
};

template <>
struct Serializable<int16_t> {
    template <typename Serializer>
    static void serialize(const int32_t &value, Serializer &serializer) {
        serializer.serialize_i32(value);
    }
};

template <>
struct Serializable<int32_t> {
    template <typename Serializer>
    static void serialize(const int32_t &value, Serializer &serializer) {
        serializer.serialize_i32(value);
    }
};

template <>
struct Serializable<int64_t> {
    template <typename Serializer>
    static void serialize(const int64_t &value, Serializer &serializer) {
        serializer.serialize_i64(value);
    }
};

template <>
struct Serializable<int128_t> {
    template <typename Serializer>
    static void serialize(const int128_t &value, Serializer &serializer) {
        serializer.serialize_i128(value);
    }
};

// Must be defined after (u)int128_t.
template <class... Types>
struct Serializable<std::tuple<Types...>> {
    template <typename Serializer>
    static void serialize(const std::tuple<Types...> &value,
                          Serializer &serializer) {
        std::apply(
            [&serializer](Types const &... args) {
                (Serializable<Types>::serialize(args, serializer), ...);
            },
            value);
    }
};

template <class... Types>
struct Serializable<std::variant<Types...>> {
    template <typename Serializer>
    static void serialize(const std::variant<Types...> &value,
                          Serializer &serializer) {
        serializer.serialize_variant_index(value.index());
        std::visit(
            [&serializer](const auto &arg) {
                using T = typename std::decay<decltype(arg)>::type;
                Serializable<T>::serialize(arg, serializer);
            },
            value);
    }
};

/* ---------- */

template <typename T>
struct Deserializable<std::unique_ptr<T>> {
    template <typename Deserializer>
    static std::unique_ptr<T> deserialize(Deserializer &deserializer) {
        return std::make_unique<T>(
            Deserializable<T>::deserialize(deserializer));
    }
};

template <typename T, typename Allocator>
struct Deserializable<std::vector<T, Allocator>> {
    template <typename Deserializer>
    static std::vector<T> deserialize(Deserializer &deserializer) {
        std::vector<T> result;
        uint32_t len = deserializer.deserialize_len();
        for (uint32_t i = 0; i < len; i++) {
            result.push_back(Deserializable<T>::deserialize(deserializer));
        }
        return result;
    }
};

template <typename T, std::size_t N>
struct Deserializable<std::array<T, N>> {
    template <typename Deserializer>
    static std::array<T, N> deserialize(Deserializer &deserializer) {
        std::array<T, N> result;
        for (T &item : result) {
            item = Deserializable<T>::deserialize(deserializer);
        }
        return result;
    }
};

template <>
struct Deserializable<std::string> {
    template <typename Deserializer>
    static std::string deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_str();
    }
};

template <>
struct Deserializable<uint8_t> {
    template <typename Deserializer>
    static uint8_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_u8();
    }
};

template <>
struct Deserializable<uint16_t> {
    template <typename Deserializer>
    static uint16_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_u16();
    }
};

template <>
struct Deserializable<uint32_t> {
    template <typename Deserializer>
    static uint32_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_u32();
    }
};

template <>
struct Deserializable<uint64_t> {
    template <typename Deserializer>
    static uint64_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_u64();
    }
};

template <>
struct Deserializable<uint128_t> {
    template <typename Deserializer>
    static uint128_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_u128();
    }
};

template <>
struct Deserializable<int8_t> {
    template <typename Deserializer>
    static int8_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_i8();
    }
};

template <>
struct Deserializable<int16_t> {
    template <typename Deserializer>
    static int16_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_i16();
    }
};

template <>
struct Deserializable<int32_t> {
    template <typename Deserializer>
    static int32_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_i32();
    }
};

template <>
struct Deserializable<int64_t> {
    template <typename Deserializer>
    static int64_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_i64();
    }
};

template <>
struct Deserializable<int128_t> {
    template <typename Deserializer>
    static int128_t deserialize(Deserializer &deserializer) {
        return deserializer.deserialize_i128();
    }
};

// Must be defined after (u)int128_t.
template <class... Types>
struct Deserializable<std::tuple<Types...>> {
    template <typename Deserializer>
    static std::tuple<Types...> deserialize(Deserializer &deserializer) {
        return std::make_tuple(
            Deserializable<Types>::deserialize(deserializer)...);
    }
};

template <class... Types>
struct Deserializable<std::variant<Types...>> {
    template <typename Deserializer>
    static std::variant<Types...> deserialize(Deserializer &deserializer) {
        auto index = deserializer.deserialize_variant_index();

        using Case = std::function<std::variant<Types...>()>;

        auto make_case = [&deserializer](auto tag) -> Case {
            using T = typename decltype(tag)::type;
            auto f = [&deserializer]() {
                return std::variant<Types...>(
                    Deserializable<T>::deserialize(deserializer));
            };
            return f;
        };

        std::array<Case, sizeof...(Types)> cases = {
            make_case(std::common_type<Types>{})...};
        return cases.at(index)();
    }
};
