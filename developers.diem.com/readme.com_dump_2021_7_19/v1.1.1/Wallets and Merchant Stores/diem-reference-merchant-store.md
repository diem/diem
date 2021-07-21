---
title: "Diem Reference Merchant Store"
slug: "diem-reference-merchant-store"
hidden: false
metadata: 
  title: "Diem Reference Merchant Store"
  description: "Explore the Diem Reference Merchant Store and how you use it to test and develop merchant stores to work with the Diem Blockchain."
createdAt: "2021-02-23T00:44:02.637Z"
updatedAt: "2021-03-30T22:49:17.456Z"
---
The Diem Reference Merchant project demonstrates the possible integration of Diem payments on an online shopping website. This project is meant to be a reference you can use while building your own merchant solution.

## Introduction

You can demo the following concepts using this reference project:

* Integration of a Diem payment solution in a pre-existing website without exposing the merchant to Diem Coins or any Diem Blockchain specifics.
* Conceptual implementation of a <<glossary:Regulated VASP>> service providing such Diem integration to merchants.
* Support for Diem payments with web and mobile wallets.


## Architecture

The Diem Reference Merchant project demonstrates three main entities that would be separate in most real world scenarios: a merchant, a Regulated VASP, and a liquidity provider.

As illustrated above, this project is a collection of the following interconnected entities:

* Online Merchant: This is a demo web shop accepting Diem payments for the items it sells. It is composed of front-end, back-end, and database services.
* Regulated Virtual Asset Service Provider (VASP): This entity provides payment clearance, fiat pay-out, and refund services to the merchant. It is composed of back-end and database services.
* Liquidity Provider: This service converts (for the Regulated VASP) Diem Coin currencies to and from fiat currencies.
* Gateway: This entity manages the network traffic between the outside world and the merchant solution services.