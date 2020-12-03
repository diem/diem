## Testnet Faucet Service

Faucet service is a simple proxy server to mint coins for your test account on Testnet.
As a side effect, it is also the only way you can create an onchain account on Testnet.

> If you wonder how simple it is, check [server code](./../../docker/mint/server.py).

It's interface is very simple, fire a HTTP POST request to `http://faucet.testnet.diem.com/` with the following parameters:

| param name    | type   | description                     |
|---------------|--------|---------------------------------|
| amount        | int    | amount of coins to mint         |
| auth_key      | string | your account authentication key |
| currency_code | string | the currency code, e.g. XDX     |

Server will start a sub-process to submit a mint coin transaction to Testnet, and return the next new account sequence for account `000000000000000000000000000000DD`.

For example, you can have something like the followings:

```Java
private static String SERVER_URL = "http://faucet.testnet.diem.com/";

public static long mintCoinsAsync(long amount, String authKey, String currencyCode) {
    HttpClient httpClient = HttpClient.newHttpClient();

    URI uri = URI.create(SERVER_URL + "?amount=" + amount + "&auth_key=" + authKey + "&currency_code=" + currencyCode);
    HttpRequest httpRequest = HttpRequest.newBuilder()
            .uri(uri)
            .POST(HttpRequest.BodyPublishers.noBody())
            .build();
    int retry = 3;
    for (int i = 0; i <= retry; i++) {
        try {
            HttpResponse<String> resp = httpClient.send(httpRequest, HttpResponse.BodyHandlers.ofString());
            if (resp.statusCode() != 200) {
                if (i < retry) {
                    waitAWhile();
                    continue;
                }
                throw new RuntimeException(resp.toString());
            }
            return Long.parseLong(resp.body());
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }
    throw new RuntimeException();
}

private static void waitAWhile() {
    try {
        Thread.sleep(500);
    } catch (InterruptedException e) {
        throw new RuntimeException(e);
    }
}
```

As the server side code is simple and can only handle 1 request per second, you may face some errors, hence above Java code has simple retry logic.

It is an async process, hence when the client receives a response, the coins are not in the account yet, client needs to wait for the transaction to be executed.
The next new account sequence is provided for you to use call [get_account](method_get_account.md) and wait for the new sequence to show up.
If you tried to call [get_account_transaction](method_get_account_transaction.md) to get the mint transaction, you should wait for the `respond account sequence - 1`.

For example, the following code calls to [get_account_transaction](method_get_account_transaction.md) to wait for minting coins transaction executed.

```Java

private static final long DEFAULT_TIMEOUT = 10 * 1000;
private static String DD_ADDRESS = "000000000000000000000000000000DD";

public static void mintCoins(Client client, long amount, String authKey, String currencyCode) {
    long nextAccountSeq = mintCoinsAsync(amount, authKey, currencyCode);
    Transaction txn = null;
    try {
        txn = client.waitForTransaction(DD_ADDRESS, nextAccountSeq - 1, false, DEFAULT_TIMEOUT);
    } catch (Exception e) {
        throw new RuntimeException(e);
    }

    if (txn == null) {
        throw new RuntimeException("mint coins transaction does not exist / failed, sequence: "+nextAccountSeq);
    }
    if (!txn.isExecuted()) {
        throw new RuntimeException("mint coins transaction failed: " + txn.toString());
    }
}

```
