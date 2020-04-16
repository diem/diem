import java.util.Vector;

public class HelloWorld {

    public static void main(String[] args) {
        Vector<Byte> bytes = new Vector<Byte>();
        Test.AccountAddress address = new Test.AccountAddress();
        address.value = bytes;

        Test.AccessPath path = new Test.AccessPath();
        path.address = address;
        path.path = new Vector<Byte>();
        System.out.println(path);
    }

}
