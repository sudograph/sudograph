import Text "mo:base/Text";
import Map "mo:base/HashMap";
import Option "mo:base/Option";

actor Motoko {
    let message_store = Map.HashMap<Text, ?Message>(10, Text.equal, Text.hash);

    type Message = {
        id: Text;
        text: Text;
    };

    public query func customGet(id: Text): async ?Message {
        return Option.flatten(message_store.get(id));
    };

    public func customSet(id: Text, text: ?Text): async Bool {
        let message: ?Message = switch (text) {
            case null null;
            case (?text_value) Option.make({
                id;
                text = text_value;
            });
        };
        
        message_store.put(id, message);

        return true;
    };
}