---
title: "Test the local merchant store"
slug: "local-web-reference-merchant"
hidden: false
metadata: 
  title: "Test the local merchant store"
  description: "Test the local web version of the Diem Reference Merchant project."
createdAt: "2021-02-11T01:09:09.161Z"
updatedAt: "2021-03-30T23:02:55.905Z"
---
## Overview

The merchant store web UI currently supports the demonstration of the Diem payment integration scenario where a direct payment under US$1,000 is made. 

### Solutions stack

The web UI has been developed using the following tools:

#### Front-end

| | |
|---- | ---- | ----|
| Programming languages | TypeScript, HTML, Sass |
| Package manager | Yarn |
| UI framework | React |
| Components and layout framework | Bootstrap |
| Testing framework | Jest |
| HTTP client | Axios |


#### Back-end

| | |
| --- | ---- |
| Programming languages | Python |
| Package manager | pipenv |
| Web framework | Flask |
| Testing framework | pytest |
| HTTP client | requests |


### Assumptions

The Diem Reference Merchant implementation makes several assumptions for simplicity:

* None of the products are real and no actual purchases are made.
* The checkout process focuses on payment and bypasses common steps such as shipping details.
* Users can “buy” products in the merchant online store without any registration or authentication.
* The merchant’s payment management UI is accessible without any authentication.

The main purpose of this implementation is to demonstrate common use cases when working with Diem Coin currencies, and to show directions for further development. To make the important concepts more clear for the readers, the implementation is deliberately simplified and avoids some of the complexities of real world, production-grade financial software.


[block:callout]
{
  "type": "danger",
  "body": "This code is to be used as a reference only. Never use this code in production!"
}
[/block]
## Test the local merchant store

### Direct payment under US$1,000
Direct payments under US$1,000 do not require any special AML procedures and are processed immediately.

1. Open the merchant store.
2. Choose a product under US$1,000 and click “Buy now.” 
[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/6ea498d-merchant-buy.svg",
        "merchant-buy.svg",
        679,
        694,
        "#000000"
      ],
      "caption": "Figure 1.0 List of products to purchase"
    }
  ]
}
[/block]
3. The checkout dialog window will open. 
4. Select the desired Diem Coin currency.
5. The calculated price in the selected Diem Coin currency will be displayed. 
[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/e8d52aa-merchant-checkout.svg",
        "merchant-checkout.svg",
        830,
        863,
        "#000000"
      ],
      "caption": "Figure 1.1 Checkout page"
    }
  ]
}
[/block]
6. Scan the QR code or copy the deep-link into a Diem wallet of your choice.
7. Complete the payment in the wallet.
8. The checkout dialog will confirm payment success.

Notice the payment ID link at the bottom of the dialog. It can be used to access the payment management page.