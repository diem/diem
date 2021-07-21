---
title: "Use the admin dashboard"
slug: "reference-wallet-admin-dash"
hidden: false
metadata: 
  title: "Use the admin dashboard"
  description: "Manage users, admins, and liquidity using the admin dashboard for the Diem Reference Wallet."
createdAt: "2021-02-04T01:06:54.395Z"
updatedAt: "2021-03-30T22:20:05.532Z"
---
## Overview

This tutorial will walk you through logging in to and using the Diem Reference Wallet admin dashboard. 

The admin dashboard allows admin users to perform several administrative tasks such as:

*   Reviewing registered users
*   Blocking users
*   Adding new admin users
*   Settling liquidity provider debt


[block:callout]
{
  "type": "info",
  "body": "* You can use the admin credentials in the tutorial below only in your local web version of the Diem Reference Wallet.\n* The admin dashboard is not accessible in the mobile app. \n* The admin user is not available for the public demo."
}
[/block]
## Log in


To access the admin dashboard:
1. On the log in page, click on 'Forgot Password'. 
2. You will be prompted to enter a user email to reset your password. Use `admin@lrw`. This admin user will always exist in the Diem Reference Wallet. 
3. The page that loads first says that the reset password email has been sent to your account. Then you will automatically be redirected to the next page, where you can enter and confirm a new password. 
4. Enter and confirm your new admin password and click 'Submit'. 
5. Log in using the username `admin@lrw` and the new password you created. 

You will be redirected to the main page of the admin dashboard, which is different from the main page of a regular user. 



## View the main page

The main page of the admin dashboard shows the total number of registered users and the total wallet balances for each Diem Coin currency. The admin dashboard also contains the following pages that can help you manage several administrative tasks: 

*   User management
*   Administrators management
*   Liquidity management
[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/58972ba-admin-dash.png",
        "admin-dash.png",
        1000,
        753,
        "#fefefe"
      ],
      "caption": "Figure 1.0 Reference Wallet admin dashboard"
    }
  ]
}
[/block]
### Manage users and admins

Manage users and admins by clicking on the user and admin management pages. Both these pages offer a list of users. Clicking “Block” button will force the user to be logged out immediately and prevent them from entering the system again.

On the administrators management page, the “Add administrator” button allows adding a new admin user to the system.


### Manage liquidity 

The liquidity management page simulates management of the wallet debt for its liquidity provider. The wallet continuously sells and buys funds from the liquidity provider. While the test Diem Coins are immediately transferred, the corresponding fiat amount is accumulated and settled manually. 

In real-world scenarios, the settlement usually involves some kind of wire transfer. In the Diem Reference Wallet, clicking “Settle” marks the debt as settled immediately (for demo purposes only).