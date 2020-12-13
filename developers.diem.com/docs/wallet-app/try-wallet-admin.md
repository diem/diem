---
id: try-wallet-admin
title: 'Reference Wallet Admin Dashboard'
sidebar_label: The Admin Dashboard
---

## Overview

The admin UI allows admin users to perform several administrative tasks:

*   Review registered users.
*   Block users.
*   Add new admin users.
*   Settle liquidity provider debt.

To access the admin UI, log in using the admin credentials `admin@drw.com`. Upon login, the user will be presented with a UI different from the regular user.


> **Note**:
> - There always exists an admin user named admin@drw.
> - The admin UI is not accessible in the mobile app.

## Main page

The main page of the admin UI shows the total number of registered users and the total wallet balances for each Diem Coin currency. From the main page, the user can browse to three secondary pages:

*   User management
*   Administrators management
*   Liquidity management

![Figure 1.0 Reference Wallet admin dashboard](/img/docs/admin-dash.png)


## User and admin management pages

Both these pages offer a list of users. Clicking “Block” button will force the user to be logged out immediately and prevent them from entering the system again.

On the administrators management page “Add administrator” button allows adding a new admin user to the system.


## Liquidity management page

This page simulates management of the wallet debt for its liquidity provider. The wallet continuously sells and buys funds from the liquidity provider. While the test Diem Coins are immediately transferred, the corresponding fiat amount is accumulated and settled manually. In real-world scenarios the settlement usually involves some kind of wire transfer. Here, for demonstration purposes, clicking “Settle” marks the debt as settled immediately.

###### tags: `wallet`
