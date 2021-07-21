---
title: "Try the Public Demo Merchant Store"
slug: "reference-merchant-public-demo"
hidden: false
metadata: 
  title: "Try the Public Demo Merchant Store"
  description: "Try the public, online demo of the Diem Reference Merchant store."
createdAt: "2021-02-11T01:06:01.457Z"
updatedAt: "2021-03-30T23:20:04.964Z"
---
The Public Demo Merchant Store (demo store) allows you to try a merchant store running on the Diem testnet. It demonstrates a simple, common use case when working with Diem Coin currencies.

The Public Demo Merchant Store is an online public demo of the Diem Reference Merchant project. 
[block:callout]
{
  "type": "warning",
  "body": "* The products in the demo store are not real and you do not make actual purchases.\n* The demo store’s checkout process focuses on payment, bypassing common steps such as asking for shipping details.\n* The demo store implementation is simplified, avoiding complexities of real-world merchant stores. For example, you can “buy” products without registration or authentication, and you can “pay” for products without authentication."
}
[/block]

[block:embed]
{
  "html": false,
  "url": "https://demo-merchant.diem.com/",
  "title": "Diem Reference Merchant",
  "favicon": null,
  "iframe": true
}
[/block]
## Purchase a product


1. Open the Public Demo Merchant Store. 
2. Click "Buy Now" for any product. The payment dialog opens with the calculated price.
3. Select a Diem Coin currency for payment. Currently, we only have one Diem Coin currency available. 
4. Scan the QR code or select a simulated Diem wallet.
5. Click "See Order Details". A sample Order Details page opens in a different browser tab or window.
The buttons "Cash Out" and "Refund", disabled here, are examples of payment management actions you could use when creating your merchant store.
6. Close the Order Details tab or window and return to the demo’s browser tab or window.
[block:callout]
{
  "type": "info",
  "body": "* Direct payments under US$1,000 do not require any special AML procedures and are processed immediately.\n* Notice the payment ID link at the bottom of the dialog. It can be used to access the payment management page."
}
[/block]