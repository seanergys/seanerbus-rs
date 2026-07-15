
@0xd6906f10b6864593;
struct Message
{
    to @0 :Uuid;
    from @1 :Uuid;
    address @2 :Uuid;
    struct Uuid
    {
    upper @0 :UInt64;
    lower @1 :UInt64;
    }
    type @3 :MessageType;
    enum MessageType
    {
        subscribe @0;
        publish @1;
        error @2;
        request @3;
        register @4;
        response @5;
    }
    payloadType @4  :UInt64;
    payload @5 :Data;
    matchingId @6 :UInt64;
}

struct ErrorPayload
{
    errorKind @0 :ErrorKind;
    enum ErrorKind {
        noSuchInterface @0;
    }
    errorStr @1 :Text;
}
