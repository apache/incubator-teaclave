def entrypoint(argv):
    assert argv[0] == 'message'
    assert argv[1] is not None
    return argv[1]
