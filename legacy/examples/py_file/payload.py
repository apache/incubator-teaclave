import mesatee

def save_file_for_task_creator(context_id, context_token):
    content = "save_file_for_task_creator"
    saved_id = mesatee.mesatee_save_file_for_task_creator(context_id, context_token, content)
    content_from_file = mesatee.mesatee_read_file(context_id, context_token, saved_id)
    if content_from_file == content: return True
    else: return False

def read_file(context_id, context_token, file_id):
    content = mesatee.mesatee_read_file(context_id, context_token, file_id)
    if content.startswith("Lorem ipsum dolor sit amet"): return True
    else: return False

def save_file_for_all_participants(context_id, context_token):
    content = "save_file_for_all_participants"
    saved_id = mesatee.mesatee_save_file_for_all_participants(context_id, context_token, content)
    content_from_file = mesatee.mesatee_read_file(context_id, context_token, saved_id)
    if content_from_file == content: return True
    else: return False

def save_file_for_file_owner(context_id, context_token, file_id):
    content = "save_file_for_file_owner"
    saved_id = mesatee.mesatee_save_file_for_file_owner(context_id, context_token, file_id, content)
    content_from_file = mesatee.mesatee_read_file(context_id, context_token, saved_id)
    if content_from_file == content: return True
    else: return False

def entrypoint(argv):
    context_id, context_token, file_id = argv
    if not read_file(context_id, context_token, file_id): return False
    if not save_file_for_task_creator(context_id, context_token): return False
    if not save_file_for_all_participants(context_id, context_token): return False
    if not save_file_for_file_owner(context_id, context_token, file_id): return False
    return True
