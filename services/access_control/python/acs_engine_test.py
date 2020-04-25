if __name__ == '__main__':
    import sys
    import os
    from acs_engine import *

    model_path = os.path.join(os.path.dirname(__file__), '../model.conf')
    test_model = open(model_path).read()
    acs_setup_model(test_model)

    FUSION_TASK               = "data_fusion"
    FUSION_TASK_PARTY_1       = "usr_party1"
    FUSION_TASK_DATA_1        = "data1"
    FUSION_TASK_PARTY_2       = "usr_party2"
    FUSION_TASK_DATA_2        = "data2"
    FUSION_TASK_SCRIPT        = "fusion_script"
    FUSION_TASK_SCRIPT_WRITER = "usr_party3"
    PUBLIC_SCRIPT             = "public_script"
    PUBLIC_SCRIPT_WRITER      = "usr_party4"

    IRRELEVANT_TASK           = "task_irrelevant"
    IRRELEVANT_PARTY          = "usr_irrelevant"
    IRRELEVANT_DATA           = "data_irrelevant"

    acs_announce_fact('task_creator', repr([FUSION_TASK, FUSION_TASK_PARTY_1]))
    acs_announce_fact('task_participant', repr([FUSION_TASK, FUSION_TASK_PARTY_1]))
    acs_announce_fact('task_participant', repr([FUSION_TASK, FUSION_TASK_PARTY_2]))

    acs_announce_fact('data_owner', repr([FUSION_TASK_DATA_1, FUSION_TASK_PARTY_1]))
    acs_announce_fact('data_owner', repr([FUSION_TASK_DATA_2, FUSION_TASK_PARTY_2]))
    acs_announce_fact('data_owner', repr([IRRELEVANT_DATA, IRRELEVANT_PARTY]))

    acs_announce_fact('script_owner', repr([FUSION_TASK_SCRIPT, FUSION_TASK_SCRIPT_WRITER]))

    acs_announce_fact('script_owner', repr([PUBLIC_SCRIPT, PUBLIC_SCRIPT_WRITER]))
    acs_announce_fact('is_public_script', repr([PUBLIC_SCRIPT]))


    assert acs_enforce_request('launch_task', repr([FUSION_TASK, set([FUSION_TASK_PARTY_1, FUSION_TASK_PARTY_2])]))
    assert not acs_enforce_request('launch_task', repr([FUSION_TASK, set()]))
    assert not acs_enforce_request('launch_task', repr([FUSION_TASK, set([FUSION_TASK_PARTY_1])]))
    assert not acs_enforce_request('launch_task', repr([FUSION_TASK, set([FUSION_TASK_PARTY_2])]))

    assert acs_enforce_request('access_data', repr([FUSION_TASK, FUSION_TASK_DATA_1]))
    assert acs_enforce_request('access_data', repr([FUSION_TASK, FUSION_TASK_DATA_2]))
    assert not acs_enforce_request('access_data', repr([FUSION_TASK, IRRELEVANT_DATA]))

    assert acs_enforce_request('access_script', repr([FUSION_TASK, PUBLIC_SCRIPT]))
    assert not acs_enforce_request('access_script', repr([FUSION_TASK, FUSION_TASK_SCRIPT]))

    acs_announce_fact('task_participant', repr([FUSION_TASK, FUSION_TASK_SCRIPT_WRITER]))
    assert acs_enforce_request('access_script', repr([FUSION_TASK, FUSION_TASK_SCRIPT]))

    acs_announce_fact('task_participant', repr([FUSION_TASK, IRRELEVANT_PARTY]))
    assert acs_enforce_request('access_script', repr([FUSION_TASK, FUSION_TASK_SCRIPT]))

    acs_announce_fact('task_creator', repr([IRRELEVANT_TASK, IRRELEVANT_PARTY]))
    acs_announce_fact('task_participant', repr([IRRELEVANT_TASK, IRRELEVANT_PARTY]))
    acs_announce_fact('task_participant', repr([IRRELEVANT_TASK, FUSION_TASK_PARTY_2]))

    assert acs_enforce_request('launch_task', repr([IRRELEVANT_TASK, set([IRRELEVANT_PARTY, FUSION_TASK_PARTY_2])]))
    assert not acs_enforce_request('access_data', repr([IRRELEVANT_TASK, FUSION_TASK_DATA_1]))
    assert acs_enforce_request('access_data', repr([IRRELEVANT_TASK, FUSION_TASK_DATA_2]))
    assert not acs_enforce_request('access_script', repr([IRRELEVANT_TASK, FUSION_TASK_SCRIPT]))
    assert acs_enforce_request('access_script', repr([IRRELEVANT_TASK, PUBLIC_SCRIPT]))

    assert acs_enforce_request('delete_data', repr([FUSION_TASK_PARTY_1, FUSION_TASK_DATA_1]))
    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_PARTY_1, FUSION_TASK_DATA_2]))
    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_PARTY_1, IRRELEVANT_DATA]))
    assert not acs_enforce_request('delete_script', repr([FUSION_TASK_PARTY_1, FUSION_TASK_SCRIPT]))
    assert not acs_enforce_request('delete_script', repr([FUSION_TASK_PARTY_1, PUBLIC_SCRIPT]))

    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_PARTY_2, FUSION_TASK_DATA_1]))
    assert acs_enforce_request('delete_data', repr([FUSION_TASK_PARTY_2, FUSION_TASK_DATA_2]))
    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_PARTY_2, IRRELEVANT_DATA]))
    assert not acs_enforce_request('delete_script', repr([FUSION_TASK_PARTY_2, FUSION_TASK_SCRIPT]))
    assert not acs_enforce_request('delete_script', repr([FUSION_TASK_PARTY_2, PUBLIC_SCRIPT]))

    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_SCRIPT_WRITER, FUSION_TASK_DATA_1]))
    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_SCRIPT_WRITER, FUSION_TASK_DATA_2]))
    assert not acs_enforce_request('delete_data', repr([FUSION_TASK_SCRIPT_WRITER, IRRELEVANT_DATA]))
    assert acs_enforce_request('delete_script', repr([FUSION_TASK_SCRIPT_WRITER, FUSION_TASK_SCRIPT]))
    assert not acs_enforce_request('delete_script', repr([FUSION_TASK_SCRIPT_WRITER, PUBLIC_SCRIPT]))
