---
title: "Manage payments"
slug: "reference-merchant-manage-payments"
hidden: false
metadata: 
  title: "Manage payments"
  description: "Use the Diem Reference Merchant store's payment management page to simulate sending refunds and payouts to your bank account."
createdAt: "2021-02-11T01:18:23.538Z"
updatedAt: "2021-03-30T23:04:48.548Z"
---
The payment management page demonstrates the actions that a merchant can perform on an existing payment. The page offers the following payment information and related actions:


* Payment information:
  * Payment ID
  * Current payment status
  * Payment status history
  * The Diem Blockchain transaction ID, if applicable
* Payment payout
* Payment refund
[block:callout]
{
  "type": "info",
  "body": "The following images (and the local reference merchant project) have payment management actions (payout and refund) currently disabled as part of the public merchant demo."
}
[/block]

[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/9c65c8e-merchant-payment-mgmt1.svg",
        "merchant-payment-mgmt1.svg",
        639,
        872,
        "#000000"
      ],
      "caption": "Figure 1.0 Payment management page with order details"
    }
  ]
}
[/block]

[block:image]
{
  "images": [
    {
      "image": [
        "https://files.readme.io/698831d-merchant-payment-mgmt2.svg",
        "merchant-payment-mgmt2.svg",
        702,
        735,
        "#000000"
      ],
      "caption": "Figure 1.1 Payment management page with cashout and refund buttons"
    }
  ]
}
[/block]
### Payment payout
This action simulates the conversion of Diem Coins into a fiat currency and the transfer to the merchant's bank account. The action focuses on demonstrating the actions involving Diem Coins and doesn’t perform any actual fiat transfers.

### Payment refund
This action simulates refunding a cleared payment. The action creates a transaction sending the paid Diem Coins back to the wallet that originally sent the payment.