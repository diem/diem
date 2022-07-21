---
title: "Manage payments"
slug: "reference-merchant-manage-payments"
hidden: false
---
The payment management page demonstrates the actions that a merchant can perform on an existing payment. The page offers the following payment information and related actions:


* Payment information:
  * Payment ID
  * Current payment status
  * Payment status history
  * The Diem Blockchain transaction ID, if applicable
* Payment payout
* Payment refund

<BlockQuote type="info">
The following images (and the local reference merchant project) have payment management actions (payout and refund) currently disabled as part of the public merchant demo.
</BlockQuote>

![Figure 1.0 Payment management page with order details](/img/docs/merchant-payment-mgmt1.svg)
<small className="figure">Figure 1.0 Payment management page with order details</small>


![Figure 1.1 Payment management page with cashout and refund buttons](/img/docs/merchant-payment-mgmt2.svg)
<small className="figure">Figure 1.1 Payment management page with cashout and refund buttons</small>

### Payment payout
This action simulates the conversion of Diem Coins into a fiat currency and the transfer to the merchant's bank account. The action focuses on demonstrating the actions involving Diem Coins and doesn’t perform any actual fiat transfers.

### Payment refund
This action simulates refunding a cleared payment. The action creates a transaction sending the paid Diem Coins back to the wallet that originally sent the payment.

<br/>
<ThankFirst/>
